use super::pipeline_setup::{
    build_stream_config, report_ethercat_credit, require_unlimited_config_jerk,
};
use super::{PyMotionEngine, planner_api::require_supported_jerk_override};
use crate::config::PlannerConfig;
use crate::lock_ext::LockExt;

#[test]
fn ethercat_credit_waits_for_delivery_and_the_laggard_motor_playback() {
    use ethercat_setpoint::setpoint_fill::{CLOCK_FREQ_HZ, ChainFiller, LaneSpec};
    use std::sync::{Arc, Mutex};
    use trajectory::{ClockedMotorSpan, ContinuousAxis, MotorGroup, MotorSpan, MotorTerm};

    let signal = Arc::new(
        MotorSpan::try_new(
            Arc::from([MotorGroup::Independent(MotorTerm {
                source_axis: 2,
                axis: ContinuousAxis::Hold {
                    position: 0.0,
                    t_start: 0.0,
                    t_end: 0.001,
                },
                scale: 1.0,
            })]),
            0.0,
            0.001,
            0,
            0,
            true,
        )
        .unwrap(),
    );
    let view =
        ClockedMotorSpan::try_new(signal, 0.0, 0.001, 0.0, 0.001, 0.0, CLOCK_FREQ_HZ).unwrap();
    let end = view.end_clock;
    let mut chain = ChainFiller::new(
        &[
            LaneSpec {
                axis: 2,
                cmd_counts_per_mm: 1000.0,
                ff_lead_ns: 0,
            },
            LaneSpec {
                axis: 2,
                cmd_counts_per_mm: 1000.0,
                ff_lead_ns: 0,
            },
        ],
        None,
        250_000,
        0,
    );
    chain.observe_grid(0, 0).unwrap();
    chain.push_spans(2, &[view]).unwrap();
    let output = chain.pending_sample_runs().unwrap().unwrap();
    let filler = Arc::new(Mutex::new(chain));
    let (tx, rx) = crossbeam_channel::unbounded();
    let report = |clocks: &[u64]| {
        report_ethercat_credit(&tx, 4, &[2, 2], &filler, clocks).unwrap();
        let crate::pump::PumpMsg::Heartbeat(credit) = rx.recv().unwrap() else {
            panic!("view credit");
        };
        credit
    };
    let pending = report(&[end, end]);
    assert_eq!(pending.consumed_counts, Some(vec![1]));
    assert_eq!(
        pending.retired_counts,
        vec![0],
        "unsent output cannot retire on an advanced playhead"
    );
    assert!(filler.lock_ok().acknowledge_sample_runs(&output));
    assert_eq!(report(&[end, end - 1]).retired_counts, vec![0]);
    let played = report(&[end, end]);
    assert_eq!(played.axes, vec![2]);
    assert_eq!(
        played.retired_counts,
        vec![1],
        "two motor slots retire one logical view"
    );
    assert!(report_ethercat_credit(&tx, 4, &[2, 2], &filler, &[end]).is_err());
}

#[test]
fn stream_config_accepts_unlimited_jerk() {
    let mut cfg = PlannerConfig::default();
    cfg.cartesian.max_jerk = f64::INFINITY;

    assert!(build_stream_config(&cfg).is_ok());
}

#[test]
fn stream_config_rejects_finite_jerk() {
    let mut cfg = PlannerConfig::default();
    cfg.cartesian.max_jerk = 100_000.0;

    assert_eq!(
        require_unlimited_config_jerk(cfg.cartesian.max_jerk),
        Err(
            "finite [printer] max_jerk is not supported by the continuous trajectory pipeline; set max_jerk: 0"
        )
    );
    assert!(build_stream_config(&cfg).is_err());
}

#[test]
fn jerk_override_accepts_none_and_positive_infinity() {
    let engine = PyMotionEngine::new();

    engine.set_jerk_override(Some(f64::INFINITY)).unwrap();
    assert_eq!(
        engine.planner_config.lock_ok().runtime_caps.jerk_override,
        Some(f64::INFINITY)
    );

    engine.set_jerk_override(None).unwrap();
    assert_eq!(
        engine.planner_config.lock_ok().runtime_caps.jerk_override,
        None
    );
}

#[test]
fn jerk_override_rejects_every_finite_value() {
    let engine = PyMotionEngine::new();

    for jerk in [0.0, 1.0, -1.0] {
        assert_eq!(
            require_supported_jerk_override(Some(jerk)),
            Err("finite jerk overrides are not supported by the continuous trajectory pipeline")
        );
        assert!(engine.set_jerk_override(Some(jerk)).is_err());
        assert_eq!(
            engine.planner_config.lock_ok().runtime_caps.jerk_override,
            None
        );
    }
}

#[test]
fn jerk_override_rejects_other_non_finite_values() {
    let engine = PyMotionEngine::new();

    for jerk in [f64::NEG_INFINITY, f64::NAN] {
        assert_eq!(
            require_supported_jerk_override(Some(jerk)),
            Err("jerk override must be positive infinity or None")
        );
        assert!(engine.set_jerk_override(Some(jerk)).is_err());
        assert_eq!(
            engine.planner_config.lock_ok().runtime_caps.jerk_override,
            None
        );
    }
}
