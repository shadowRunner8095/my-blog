# How I Did the Navbar

## Disclaimer

I’m not versed in UX/UI, product design, or anything fancy like that.  
What I do here is just for fun.

## Inspiration

See, I really love the 80s vibe and aesthetics of that time, but my page didn’t reflect it at all.

I shared the website link with a dear friend, and she pointed out how the site looked aesthetically — and honestly, I agreed with her. I was like, “But this is an informative blog! The purpose isn’t aesthetics...”  

Well, even if this website is mainly informative and the repo is just a place to create and share open source projects, I hate being boring to myself.  
Yeah, I want to see something that *yells* — this is me.

Another thing I believe is that patterns are meant to be broken *organically*. Meaning: if you  
- \- Understand why the pattern exists in the first place  
- \- Know in what situations it’s most useful  
- \- Get the design principles behind the pattern  
- \- Are aware of the trade-offs it involves  

Then you can keep the best parts of that pattern and “break” others, merging it with parts or principles from different patterns.

## Breaking the Horizontal/Vertical Pattern

Most navbars are horizontal on desktop and lateral collapsible menus on mobile. I think the reason is that we need to use space as efficiently as possible, minimizing wasted areas.  
This is something we also see in printed media, especially in formal contexts — and it makes a lot of sense.

Still, if you look at printed media like posters or movie promos, they often use diagonals to break the monotony of linear text. Also, some textbooks — like the *Head First* series — have side notes in different orientations. I take notes like that myself and I like it.

So I decided to try a diagonal navbar instead of a horizontal or vertical one.

I really like David Bowie’s Ziggy Stardust persona, so that’s why I want this navbar to be **Z-shaped**.

For mobile, since space is limited, I’ll stack each item vertically — but I’m still going to rotate them.

## Adopting a Menu-Like Pattern

I think having a navbar that’s too different might distract users, so I’m experimenting with a collapsed-by-default navbar that can be toggled open or closed by a circular button inside a rotated lateral container.

## Animation

If the navbar is hidden by default, it definitely needs animation.  
For now, a fade-in and fade-out transition will do (80s, remember?).

## Pixel Art

More photos on the site mean slower loading, so instead of photos I’ll use icons.  
But some things can’t be expressed with just an icon and need more composition — for those cases, I’ll use pixel art (retro vibes, right?).

## Randomness

The website should work without JavaScript at all (progressive enhancement). This covers users with JS disabled or cases where we disable JS for performance.

But things delegated to JS should be non-critical — like showing random color combinations or random pixel art.

I really like chaos, so later versions might randomize some navbar appearance parameters each time the user shows or hides it.

## Simplicity

Some navbars are like full decks or site maps, which makes sense sometimes. But for this blog, the fewer navigation items, the better.

It will only serve to navigate to important pages like Home, Content Index, and similar.  
In the future, the theme switcher will be there, but I’m considering *not* putting the search bar in the navbar — instead, it could be an access link to a search page if JS isn’t available, and a modal search if JS *is* available.

## The Search Functionality

That’s a discussion for another time. I still need to decide if I’m going to “peacock” that part or keep it simple.

## Progressive Enhancement

The menu behavior should be implemented *without* JavaScript. You can learn more about that [here](https://www.freecodecamp.org/news/what-is-progressive-enhancement-and-why-it-matters-e80c7aaf834a/).

## The Implementation

To simplify the first iteration, we can break the navbar into two problems:  
- \- The menu open/close button  
- \- The navbar itself  

The first problem is already solved in another article by generalization, so we focus now on the second:

- \- The Z form  
- \- The diagonal direction  
- \- The fade-in, fade-out animation  

Animations can take some time, so the first two problems seem approachable. Let’s try the first one.

The Z form can be approached by having two containers, one below the other — but this means we have to manually decide which items go into the first div and which go into the second each time we add a nav item.

Creating a vision with one container that auto-layouts the items diagonally seems difficult — this is a constraint. But on the plus side, this manual separation is handy.  
I could even add more container layers below, like a stairway rather than a strict Z, to create natural separation. Too many layers, though, would become cumbersome, so there’s a logical limit.

For more complex cases, each container could be a collapsible group that shows more items when clicked — but that’s a future vision.

For the diagonal direction, a simple CSS transform with `rotate()` will do.

An example of overlapping two divs can be done with CSS grid and translate like this:

```html
<div class="grid grid-cols-2 grid-rows-2">
  <div class="w-[110%] bg-fuchsia-200">Container 1</div>
  <div class="col-end-3 row-end-3 w-[110%] bg-red-300 -translate-y-2">Container 2</div>
</div>

```
The rest is implementation details like colors and rotation angles — you can see that in the repository code.

Finally, we need to tell the Jinja template what to put in each container.

Since we only have two containers right now, we can abstract something like this:

```ts
// Yes, I’m using a TS interface for brevity to communicate

interface AnchorData {
  url: string;
  textContent: string;
}

interface Data {
  leftContainer: AnchorData[];
  rightContainer: AnchorData[];
}

```
The final result is the navbar you can use rigth now, but it is too dependant of the tailwind
css which because of a network error cannot be fetched so we have a long raod ahead solving those
challenges.

Want to see the code? It's [here](https://github.com/shadowRunner8095/my-blog/blob/391f59b00d2238fb194f25acb57ede3bd107ad12/templates/navbar.html)