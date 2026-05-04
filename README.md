# CSC372-Final-Project
For our final project, Trinity and I are learning Rust. The first program we will make is required for our class, the second program will be a project that we think will be interesting.

Google Doc Access to our Language Study: https://docs.google.com/document/d/1brCZOWvjgG3KfLm2vnwKCFu1zavV5Dc-oWKo0MbDC6c/edit?usp=sharing

The Program we decided to make for our final project was a 2d platformer. We chose to do this because
we are both interested in game design and exploring the capabilities of the Rust language when it comes
to games. For the game engine we used Bevy and Rapier 2d physics. The game is a side scroller that 
has a camera which begins moving as soon as the game starts, if the player gets left behind by the camera then they lose the game.
For movement the basic mechanics are left, right, and jump. The platforms are randomly generated as the camera scrolls to keep
the game going forever until the player loses. Bevy is used to create the specific aspects of our game like the window, camera,
platforms, floor, player. The Rapier physics are added so that that the actual gameplay can be added such as movement, jumping, falling,
and hit boxes so that the player can jump on boxes and not just go through them. This project taughts us a lot about how complex game design
can be while also giving us the ability to code with Rust and implement dependencies that are commonly used with the language.
