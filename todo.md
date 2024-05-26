# To Dos

- [ ] Graphics
  - [ ] Skybox
  - [ ] Bloom
- [ ] Audio Engine
  - [X] [cpal](https://github.com/RustAudio/cpal) low level cross platform
    - Can build output stream react to inputs?
    - Likely need an audio thread running that receives messages
  - [ ] Current handling is Naive, uses a lot of memory and causes a slow startup
- [ ] Misc
  - [ ] Add menus
  - [ ] Add scenes
  - [ ] Replace unwraps with proper error handling
  - [ ] Add death state
    - Add audio effect to music like slow down
    - Wait for input and restart song