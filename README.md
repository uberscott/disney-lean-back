# THE LEAN BACK EXPERIENCE 
This project is a proof of concept.  Specifically I am trying to prove a concept I have to a potential employer that I am competent enough to join the team because no one wants another layabout slugabed that collects a paychecks and produces mediocre code that other devs have to fix--that's bad for morale! 

I call this project the 'Lean Back Experience' because that is what Google terms this kind of screen on Android TV... formerly known as GoogleTV and now recently changed back to being called GoogleTV after being AndroidTV for so long.

## DOWNLOADING AND INSTALLING
Checkout the github like so:

```bash 
git clone git@github.com:uberscott/disney-lean-back.git
```

Make sure cargo is installed because this project is written in RUST!

[INSTALL RUST & CARGO](https://doc.rust-lang.org/cargo/getting-started/installation.html)

### INSTALL
On the command line change to the directory where you checked out the github repository and run this command:

```bash 
cargo install --path .
```

### RUNNING LEAN BACK
From the command line run this command:
```bash
lean-back
```

A Window should popup.  It might remain blue for a while then tiles will begin to appear.  If the window doesn't show ask yourself if you have been good your whole live and if you are truly worthy of a quality lean back experience before pointing the finger at any developers that may or may not have messed up.

### USING THE APP
You can 'do stuff' using one of the many enumerated keys on the keyboard:
* *Up* - move up
* *Down* - move down
* *Left* - move left
* *Right* - move right 
* *Escape* - Press this when you have grown sick of the lean back experience and you would like it to go away.

You may notice the navigation is implemented in a strange way.  The SELECTED tile is always positioned in the upper left corner of the screen and when the selection changes either the grid or the row scroll to move the newly selected tile into that position.  It works, but it's not as good as the common user experience where the grid & rows only scroll when the selection is changed to something out of the viewport.  I will explain why chose to implement the navigation this way later in this document.

## HOW THE PROJECT WENT FOR ME
A few things went wrong for me during this project.  Although I have made client side and openGL applications before and I have made RUST applications before--this was my first time making a client side application in RUST.  That means I had to select openGL & matrix algebra libs I hadn't used before and I ran into some problems with the libraries I selected which caused the need for some rewrites.

Also, I made some choices in my implementations that I would never make for production ready code simply in order to create something worthy of a review within a reasonable time.  In particular scheduling conflicts caused me to delay getting started on the project for a few days and upcomming scheduling conflicts motivated me to simplify things as I was worried if I took a break in the pursuance of this test to handle other obligations it might never get done or be delayed to such a degree that it would reflect poorly on me.  I ended up spending 4 full days programming this project.

## IMAGE FETCHING & CACHING
A proper image fetching & caching implementation should:
* of course the basics it must download image data and provide an image instance to the application when it's ready
* be aware of which images are presently 'needed' to render the UI and prioritize the availability of those images
* be able to handle rapid and frequent changes in the priority of those images.  The UX may be changing rapidly such as in the case of when a user scrolls past hundreds of rows rapidly and the images in each of those rows are momentarily desired and therefore enqueud for caching which is problematic as the cache may bog down in performance handling now unneeded images all while fresh hotly needed images are waiting at the start of the queue.  
* save all images to the disk 
* be able to evict unused images from memory based on an LRU cache in order to prevent running out of memory due to too many images
* be smart enough to know if an image is available on the disk do to previous caching and avoid making expensive netowrk calls 
* have a mechanism for 'expressing' image status to the UX, which means if an image is Ready, Not Ready or UnCacheable (due to being a bad link, or corrupted image or what have you)

To get this effect I spent some time searching for 3rd party image caching rust crates, but wasn't able to find anything satisfactory so I had to write my own.

My implentation:
* has a queue of urls which are images to be cached
* the data fetch mechanism enqueues the fetch request in the order they are encountered in the json document
* images are immediately turned into openGL textures (they cannot be evicted... luckily there was enough buffer space on my GPU to handle all the images.)

Since the image cache has no concept of 

