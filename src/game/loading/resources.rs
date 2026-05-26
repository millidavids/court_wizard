// `LoadingProgress` and `TOTAL_LOADING_FRAMES` used to enforce a minimum
// loading-frame floor so the loading screen had something to show. With
// bulk-processed spawn queues in both SP (`process_spawn_queue`) and MP
// (`process_mp_spawn_queue`), loading completes in ~3 imperceptible frames
// and the floor — and the screen — are gone.
//
// If a future long-running load reintroduces a progress concept (cinematics,
// streaming asset packs), reach for an explicit task-count percentage rather
// than a frame-count floor.
