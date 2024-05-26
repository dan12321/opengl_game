# To Dos

- [ ] Graphics
  - [ ] Skybox
  - [ ] Bloom
- [ ] Audio Engine
  - [X] [cpal](https://github.com/RustAudio/cpal) low level cross platform
    - Can build output stream react to inputs?
    - Likely need an audio thread running that receives messages
  - [ ] Speed up audio load time
  - [X] Fix speed higher than expected (samples per second should respect config)
  - [ ] Pitch changes through the song in a repeatable way
- [ ] Misc
  - [ ] Add menus
  - [ ] Add scenes
  - [ ] Fix bug in position update
  - [ ] Replace unwraps with proper error handling
  - [ ] Add death state
    - Add audio effect to music like slow down
    - Wait for input and restart song