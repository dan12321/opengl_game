# To Dos

- [ ] Graphics
  - [ ] Skybox
  - [ ] Bloom
  - [X] Directional Lighting
  - [ ] Shadows
- [ ] Audio Engine
  - [ ] Speed up audio load time
    - Option 1 is load chunks as needed. This will also stop large tracks from
      filling up memory
    - Option 2 add loading stage while resources are processed
      (May be wanted later anyway but the current loading time for just a 1min wav is still poor)
    - Option 3 profile where time is being spent to try and identify performance mistakes
  - [ ] Changing audio source causes a crash
  - [ ] Add stereo support
  - [ ] Add mp3 support
- [ ] Scene stuff
  - [ ] Add menus
  - [ ] Add scenes
    - [ ] Use a stack like structure where scene resources can be popped off and
          push on the resources for the next scene
- [ ] Control/feel
  - [ ] Add input buffer (not sure needed yet)
  - [ ] Adjust the linear interpolate time based on gaps between beets. Currently
        hard to leave late enough to not hit beat in next lane while not hitting
        current lane
- [ ] Gameplay
  - [ ] Move camera slightly with movement
  - [ ] Tilt camera up slightly so it looks ahead of the player
  - [ ] Make spacing of tiles match beat (maybe load separate to plane)
- [ ] Misc
  - [ ] Replace unwraps with proper error handling
  - [ ] Look into porting to android
  - [ ] Make it so the plane moves and repeats
  - [ ] Refactor to use standard model naming
    - Node, mesh, scene
  - [ ] Refactor to have resource loader that owns resources and lets other
        areas reference them
      - Load maps (add resource information to maps)
      - Load audio
      - Load models
      - Maybe load scenes onto stack like structure, can refactor to this later
  - [ ] Can define vertex and use vec of vertices instead of keeping track of offsets. Can even use offsetof to get the offsets when reading
