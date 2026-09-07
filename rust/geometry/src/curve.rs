#[must_use]
pub fn to_collinear_bezier(start: [f64; 3], end: [f64; 3]) -> [[f64; 3]; 4] {
    let d = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
    let p1 = [
        start[0] + d[0] / 3.0,
        start[1] + d[1] / 3.0,
        start[2] + d[2] / 3.0,
    ];
    let p2 = [
        start[0] + 2.0 * d[0] / 3.0,
        start[1] + 2.0 * d[1] / 3.0,
        start[2] + 2.0 * d[2] / 3.0,
    ];
    [start, p1, p2, end]
}

#[derive(Debug, Clone, Copy)]
pub struct BezierHandles {
    pub i: f64,
    pub j: f64,
    pub p: f64,
    pub q: f64,
}

#[must_use]
pub fn g5_control_points(
    start: [f64; 3],
    handles: BezierHandles,
    displacement: [f64; 3],
) -> [[f64; 3]; 4] {
    let [dx, dy, dz] = displacement;
    let end = [start[0] + dx, start[1] + dy, start[2] + dz];
    let p1 = [
        start[0] + handles.i,
        start[1] + handles.j,
        start[2] + dz / 3.0,
    ];
    let p2 = [
        end[0] + handles.p,
        end[1] + handles.q,
        start[2] + 2.0 * dz / 3.0,
    ];
    [start, p1, p2, end]
}

#[must_use]
pub fn g51_control_points(
    start: [f64; 3],
    i: f64,
    j: f64,
    dx: f64,
    dy: f64,
    dz: f64,
) -> [[f64; 3]; 4] {
    let q0 = start;
    let q1 = [start[0] + i, start[1] + j, start[2] + dz / 2.0];
    let q2 = [start[0] + dx, start[1] + dy, start[2] + dz];
    let elevate = |a: [f64; 3], mid: [f64; 3]| {
        [
            a[0] + 2.0 / 3.0 * (mid[0] - a[0]),
            a[1] + 2.0 / 3.0 * (mid[1] - a[1]),
            a[2] + 2.0 / 3.0 * (mid[2] - a[2]),
        ]
    };
    [q0, elevate(q0, q1), elevate(q2, q1), q2]
}

#[cfg(test)]
mod tests;
