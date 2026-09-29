# Kerr free-fall lattice

This is a visualization of a chosen family of freely falling **massive test
observers**, not a calculation mesh, a material model of moving space, or a
Euclidean global isometric embedding. It has no coupling to photon initial
conditions, detector events, standard archives, IDs, or reproduction.

## Physical definition

The project uses (-,+,+,+), G=c=M=1 and the exterior Boyer–Lindquist chart.
At every marker choose specific energy E=1 (rest at infinity), Lz=0 and
p_theta=0. The *massive* Carter invariant is
Q=p_theta²+cos²(theta)[a²(1-E²)+Lz²/sin²(theta)]=0; hence theta is constant.
Do not use the existing **null** Carter formula for these observers.
Starting the displayed lattice at finite positions selects finite-time points
of these rain geodesics; it does not mean release from rest at that radius.

`visualization/free_fall.rs::flow` calls the unchanged `Kerr::inverse` and
`Kerr::covariant`. For BL index order (t,r,theta,phi), it constructs

```text
p_t = -1, p_theta = p_phi = 0
p_r = -sqrt((-1 - g^tt) / g^rr)
u^mu = g^(mu nu) p_nu
d(r,theta,phi)/dt_BL = (u^r,u^theta,u^phi)/u^t
```

The negative root selects inward motion; u^t>0 selects future evolution.
No radial force, vortex, added angular speed or alternative metric is present.
In particular u^phi/u^t=g^tphi/g^tt=-g_tphi/g_phiphi: the original frame-dragging
arrows illustrate the same angular velocity, but are never added to this flow.

At chi=0, dr/dt=-(1-2/r)sqrt(2/r)<0 and dphi/dt=0. For positive chi,
dr/dt<0 and dphi/dt>0. The Kerr API now accepts signed chi in [-0.9999,0.9999];
+z is the reference axis and negative chi reverses angular motion. This is an
input-domain extension, not a blanket accuracy guarantee. See
[signed-spin validation](validation/SIGNED_SPIN_VALIDATION.md). This visualization's
advection algorithm is unchanged and was not separately validated at the new endpoints.

Related primary reference: Chris Doran, *A new form of the Kerr solution*,
[gr-qc/9910099](https://arxiv.org/abs/gr-qc/9910099). That paper uses a different
time coordinate and signature. This implementation keeps the project's BL
time, signature and original metric; it does not implement Doran coordinates.

## Coordinates, topology and numerical limits

Initial positions are an ordinary Cartesian-like cube [-extent,+extent]^3 with
`density` nodes per axis. The existing `from_cartesian` maps them to BL. The
initial six-neighbor lattice topology stays fixed; removed markers' edges
disappear without reconnecting or respawning. Connecting segments are coordinate
display links, not light rays, spatial geodesics or physical rods.

The unchanged display map is X=sqrt(r²+a²)sin(theta)cos(phi),
Y=sqrt(r²+a²)sin(theta)sin(phi), Z=r cos(theta). Time remains t_BL.
At one common t_BL, cached **BL** positions are interpolated before this mapping;
phi is unwrapped. There is no metric-based visual warp of positions.

Exterior termination uses r<=r_plus+0.02 M; the margin is numerical, not an
enlarged horizon. Unsupported initial points and the axis (|sin theta|<1e-8)
are excluded. RK4 trials entering the margin terminate their marker. Markers
are hidden after their last valid output sample, at most one output interval
early. Errors/nonfinite flow abort cache construction with a visible error;
they are not passed to the GPU or interpreted as black-hole physics.

The clock covers t_BL=0..80 M. Vector-field RK4 advection uses maximum step
0.05 M, cached every 0.25 M, in f64. Display interpolation is approximate.
No horizon crossing is claimed: near-horizon slowing is a BL coordinate effect.
The tests check normalization, Hamiltonian consistency, step convergence and
interpolation, not merely how plausible the animation looks.

The diagnostics expose gamma_rr, gamma_theta_theta and gamma_phi_phi by taking
the spatial block of the original covariant metric on t_BL=const. These are
coordinate components of that slice's spatial metric, not the infalling
observer's rest-space metric, and do not change marker positions. One labelled
probe shows r/theta/phi and coordinate velocities; it can disappear at the margin.

## GUI and caching

The **Kerr Free-Fall Grid (visualization only)** panel provides independent
Cartesian Reference Grid, Kerr Free-Fall Grid and Frame Dragging toggles.
All eight visibility combinations are supported. Existing reference-grid and
frame-dragging geometry/formulas are unchanged.

The free-fall panel has extent, nodes/axis, grid speed, Play/Pause and Reset.
Its clock is independent of ray/detector playback. Hide pauses grid advancement;
Reset restores every marker together and pauses. At 80 M playback stops; Play
starts a new whole-lattice cycle. No individual marker respawns.

Only the **displayed scene's** chi, free-fall extent and density invalidate the
cache. Thus a pending physical chi change does not mix new-spin markers with
old-spin rays. A separate coalescing worker with generation IDs precomputes the
cache. It never submits to the photon worker or standard auto-save path. Camera,
speed, visibility and Reset reuse the cache. UI reports cache build count/time.

During animation, retained CPU arrays interpolate cached positions, then update
one batched vertex buffer. The GPU allocation grows only when needed and is
reused. Paused camera motion uploads no lattice vertices; camera during playback
does the same animation upload that would occur with a stationary camera.
Photon trajectory caches are independent and unchanged.

Default free-fall display is OFF; extent=10 M, density=9, speed=3 M/s.
Limits: extent 3..100, density 3..17, speed 0.01..100. These are display resource
limits. View settings are optional JSON fields under `view.free_fall`, with
backward-compatible defaults. No field is added to common.json or Parquet.

## Where to modify

- `src/session.rs::FreeFallConfig`: independent display settings/defaults.
- `src/visualization/free_fall.rs`: metric-derived flow and cached lattice.
- `src/visualization/free_fall_ui.rs`: isolated worker, clock and controls.
- `src/visualization/renderer.rs::update_free_fall`: reusable animation buffer.
- `tests/free_fall.rs`: seven numerical/cache/isolation regression tests.

Changing the flow requires these tests and the existing physics suite to pass.
The null solver and its initial conditions are deliberately not modified.
