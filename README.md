# THE LEAN BACK EXPERIENCE 
This project is a proof of concept.  Specifically I am trying to prove a concept I have to a potential employer that I am competent enough to join the team because no one wants another layabout slugabed that collects paychecks and produces mediocre code that other devs have to fix--that's bad for morale! 

I call this project the 'Lean Back Experience' because that is what Google terms this kind of screen on Android TV... formerly known as GoogleTV and now recently changed back to being called GoogleTV after being AndroidTV for so long.

## DOWNLOADING AND INSTALLING
Checkout the github like so:

```bash 
git clone https://github.com/uberscott/disney-lean-back.git
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

A Window should popup.  It might remain blue for a while then tiles will begin to appear.  If the window doesn't show, ask yourself if you have been good your whole life and if you are truly worthy of a quality lean back experience before pointing the finger at any developers that may or may not have messed up.

### USING THE APP
You can 'do stuff' using one of the many enumerated keys on the keyboard:
* **Up** - move up
* **Down** - move down
* **Left** - move left
* **Right** - move right 
* **Escape** - Press this when you have grown sick of the lean back experience and you would like it to go away.

You may notice the navigation is implemented in a strange way.  The SELECTED tile is always positioned in the upper left corner of the screen and when the selection changes either the grid or the row scroll to move the newly selected tile into that position.  It works, but it's not as good as the common user experience where the grid & rows only scroll when the selection is changed to something out of the viewport.  I will explain why I chose to implement the navigation this way later in this document.

## EXPLANATIONS & RATIONALES
### HOW THE PROJECT WENT FOR ME
A few things went wrong for me during this project.  Although I have made client side and openGL applications before and I have made RUST applications before--this was my first time making a client side application in RUST.  That means I had to select openGL & matrix algebra libs I hadn't used before and I ran into some problems with the libraries I selected which caused the need for some rewrites.

Also, I made some choices in my implementations that I would never make for production ready code simply in order to create something worthy of a review within a reasonable time.  In particular scheduling conflicts caused me to delay getting started on the project for a few days and upcoming scheduling conflicts motivated me to simplify things as I was worried if I took a break in the pursuance of this test to handle other obligations it might never get done or be delayed to such a degree that it would reflect poorly on me.  I ended up spending 4 full days programming this project.

### IMAGE FETCHING & CACHING RATIONALE
A proper image fetching & caching implementation should:
* of course the basics it must download image data and provide an image instance to the application when it's ready
* be aware of which images are presently 'needed' to render the UI and prioritize the availability of those images
* be able to handle rapid and frequent changes in the priority of those images.  The UX may be changing rapidly such as in the case of when a user scrolls past hundreds of rows rapidly and the images in each of those rows are momentarily desired and therefore enqueued for caching which is problematic as the cache may bog down in performance handling now unneeded images all while fresh hotly needed images are waiting at the start of the queue.  
* save all images to the disk 
* be able to evict unused images from memory based on an LRU cache in order to prevent running out of memory due to too many images
* be smart enough to know if an image is available on the disk do to previous caching and avoid making expensive network calls 
* have a mechanism for 'expressing' image status to the UX, which means if an image is Ready, Not Ready or UnCacheable (due to being a bad link, or corrupted image or what have you)

To get this effect I spent some time searching for 3rd party image caching rust crates, but wasn't able to find anything satisfactory so I had to write my own.

My implementation:
* has a queue of urls which are images to be cached
* the data fetch mechanism enqueues the fetch request in the order they are encountered in the json document
* images are immediately turned into openGL textures (they cannot be evicted... luckily there was enough buffer space on my GPU to handle all the images.)

Since my image cache provides no mechanism for the UX to express what it needs it causes a somewhat chaotic (and slower) loading experience.

My reason for such a basic implementation was simply staying within the time constraints of the assignment as I think the 'proper' image cache i described would take one or two weeks alone to implement. 

### NAVIGATION RATIONALE
Yes, the navigation is unusual.  I explained earlier that it works to keep the selected tile always in the upper left corner by moving the grid or rows.  The reason I took this approach was to avoid a whole class of potential bugs.  

So the potential bugs I feared are from experience trying to determine if geometry is within the view frustum and if it isn't then in what WAY is it outside of the frustum in order to determine what action needs to be taken to get it back inside the frustum.  For example: if the last visible tile in a row is selected and the user pushes the RIGHT key, the selected tile will now be beyond the right edge of the frustum and therefore the appropriate action would be to lerp the row's X offset by a value that would make the newly selected entirely visible and with a small margin... however this gets exceptionally complicated particularly when you encounter tiles that are straddling the frustum edge.

My feeling was that a rushed implementation of the desired navigation I described would have resulted in visible bugs during the presentation creating a more frustrating experience than the solution I ultimately supplied.

If I was working on this project in a production capacity I would simply take the time to iron out all the bugs for the technique that provided the best user experience.

### MISSING FEATURE - SET TITLES
One of the requirements in the doc was to have TITLES for the sets (they are called 'sets' in the json and rendered as Rows in the UX.)

When I began this project I started with one opengl crate which I couldn't get working, then move to a crate called 'glium.'  During the process of working with glium I discovered that the maintainer discontinued the glium project, but everything was working so I kept going...

After getting the image tiles working I moved on to rendering Text using a different crate called 'glium-text.'  I basically found out that glium-text did not support the latest version of 'glium' as it had also been abandoned.  I downgraded glium to a glium-text friendly version and had to heavily refactor my code to support the older library AND after that discovered that older version of glium does not work on the latest version of my OS (or probably any updated OS.)  I rolled back my changes to the latest glium library which basically doesn't support Text anymore.

At that point I took a break trying to decide what to do and determined that in order to implement this feature I would have to select yet another openGL library and rewrite a large portion of the architecture. 

Given my time constraints I decided to leave the SET TITLES feature out.

### MISSING FEATURE -- CHOOSE TITLE
Another requirement that was not met was the ability to 'zoom' in on a tile... or choose it.  I was going to write the code to press 'space' bar and make the selected tile scale to full screen, but I just ran out of time for this feature and determined my remaining efforts would be better spent cleaning up the code, fixing all the compile warnings and writing this README document.

## WHAT I WOULD HAVE DONE IF I HAD MORE TIME
### ADDITIONAL BELLS AND WHISTLES
A few more important things not necessarily mentioned in the requirements document but important to me included: Ease In/Out Interpolation for transitions, a loading screen to be shown while the data is being fetched (instead of a blank blue screen), fade transitions on tiles when an image goes from uncached to cached

### ADDING MAGIC
Also the requirements document mentioned sprinkling in some 'magic.'  Well, wasn't able to get to the Magic part but I had some ideas I wanted to share with you:

* **Lean Back from an Alternative Dystopian Reality where Russia Won the Cold War**.  This would have included lots of propaganda messages like "Comrade, You must begin binge watching a new show in the next 24 hours or report to the bureau of streaming for mental reconditioning."  
* **Creepy Mode** strange music would play in the background and every so often a clown would peer from behind one of the tiles then dart back into hiding as if he was up to something
* **Unsafe Mode** the idea here was that the entire program would run 25% faster but there would be warnings that rapidly moving tiles could in some rare cases be ejected from the TV set and do potential harm to the viewer (as the tiles are quite sharp)
* **OMG FIRE THIS DEVELOPER IMMEDIATELY MODE!** In this mode tiles would have different personalities.  Some would spin back and forth like they were mentally addled, others would be lazy and not move into their positions at the same speed, still other tiles would bully their neighbors pushing them around and scaling up large (even when they weren't the selected tile).  It did occur to me that creating OMG FIRE THIS DEVELOPER IMMEDIATELY MODE may not be a good idea to include in a project where I was actually trying to get hired, as the mode's primary objective is to drive management crazy with range--and obviously those are the people that I want to like me... so for this one I guess it worked out that I ran out of time before getting to it.
* **Fake Badgification Mode** This would give the viewer fake badges for things they didn't really do.  Example Badge: *"As a thanks for watching so much TV we have decided to give you 5% **ADDITIONAL** happiness next month.  Have fun with all the extra happiness!"*

* **Bad Advice Mode** While the user is watching TV a ticker tape of random messages appears, always giving advice that if the viewer follows might actually cause him harm.  Examples:
  - "If you see a boa constrictor sleeping in nature, and you lie down next to it, when it wakes it will become tame and ever loyal to you"
  - "To prevent a polar bear attack, point your forearm vertically and shove it directly towards the polar bear's face.  The polar bear won't be able to open his jaw wide enough to grab hold of your forearm and they are incapable of rotating their heads... be prepared to 'dance' with the polar bear for about an hour as he tries to eat you, but rest assured he will get tired eventually and move on to bigger and better things."
  - "Native Americans discovered that one can gain immunity to poison ivy by dipping a wad of poison ivy leaves in mayonnaise and swallowing it whole.  The scientific explanation is that the mayonnaise protected the body from the poison as it was being consumed and once it was digested the body's 'poison receptors' would forever more recognize the toxins as the same that were coming from inside the body and therefore there would be no reaction.  Also: Native Americans invented Mayonnaise."

Actually upon review the document asked specifically for 'Disney magic' and the magic I was offering might not fit exactly with the brand, but hey, that's just another reason to keep Rust developers as far away from marketing as possible.

## SUMMARY
Well, I'm grateful for the opportunity to show you what I got.  I hope someone reading this will deem me worthy of joining the team and we can get to creating some really cool experiences together.

Thank You!

--Scott
