# To Dos

- [ ] Graphics
  - [ ] Skybox
  - [ ] Bloom
  - [X] Directional Lighting
  - [ ] Shadows
  - [ ] Replace temp textures
- [ ] Audio Engine
  - [X] [cpal](https://github.com/RustAudio/cpal) low level cross platform
    - Can build output stream react to inputs?
    - Likely need an audio thread running that receives messages
  - [ ] Speed up audio load time
    - Option 1 is load chunks as needed. This will also stop large tracks from
      filling up memory
    - Option 2 add loading stage while resources are processed
      (May be wanted later anyway but the current loading time for just a 1min wav is still poor)
    - Option 3 profile where time is being spent to try and identify performance mistakes
  - [X] Fix speed higher than expected (samples per second should respect config) (kinda like the pitch shift solo)
  - [X] Pitch changes through the song in a repeatable way
    - Increased precision of time variables to f64. Still slight drift between
      wav time and measured time but the rate of rate no longer changes (solving pitch).
  - [ ] Add stereo support
  - [ ] Add mp3 support
  - [X] Refactor to not use magic numbers
- [ ] Scene stuff
  - [ ] Add menus
  - [ ] Add scenes
    - [X] Change map with buttons (currently hacky)
    - [ ] Use a stack like structure where scene resources can be popped off and
          push on the resources for the next scene
    - [ ] Add loading screen and reset time
  - [X] Read map from file
  - [X] Add beet offset to map file 
- [ ] Control/feel
  - [X] Fix bug in position update (sometimes flashes in old position)
    - Was due to linear interpolation calculation
  - [ ] Add input buffer (not sure needed yet)
  - [ ] Adjust the linear interpolate time based on gaps between beets. Currently
        hard to leave late enough to not hit beat in next lane while not hitting
        current lane
- [ ] Gameplay
  - [X] Add death state
    - Add audio effect to music like slow down
    - Wait for input and restart song
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
