# To Dos

- [ ] Graphics
  - [ ] Skybox
  - [ ] Bloom
  - [ ] Directional Lighting
  - [ ] Shadows
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
  - [X] Refactor to not use magic numbers
- [ ] Scene stuff
  - [ ] Add menus
  - [ ] Add scenes
    - [X] Change map with buttons (currently hacky)
    - [ ] Use a stack like structure where scene resources can be popped off and
          push on the resources for the next scene
    - [ ] Add loading screen and reset time
  - [X] Read map from file
  - [ ] Add beet offset to map file 
- [ ] Control/feel
  - [ ] Fix bug in position update (sometimes flashes in old position)
  - [ ] Add input buffer
- [ ] Gameplay
  - [X] Add death state
    - Add audio effect to music like slow down
    - Wait for input and restart song
- [ ] Misc
  - [ ] Replace unwraps with proper error handling
  - [ ] Look into porting to android