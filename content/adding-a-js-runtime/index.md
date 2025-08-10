# Adding a JS client side runtime to my blog

Next.js, Qwik, Gatsby and other frameworks/page builder have this feature
of working like a SPA when trying to navigate to another page of the same site
after the first initial load.

This in other words means that, if you visit https://my-page, first
the prerendered html is going to download and also the common js, css
and the specific files for that page (if possible). Then if you
click a link in that page that makes you "navigate" to another 
page, actually what happens is that the JS code is in charge 
of creating in the client side, let's say https://my-page/page-2 instead
of actaully going to that page, in reallity you never leaved https://my-page
but the client code gives you that sensation avoiding some white interlude
while the new page downloads

If you go from the browser navigation box directly to https://my-page/page-2, that
is not going to happen, I mean, there is no JS already downloaded yet so
the actual navigation occurs, only you where clickin an anchor form a laready visited page
the client side navigation occurs.

I want something similar for my page, so for that the idea that I have
is to have a click listener that checks if what we clicked was an anchor
that has a data-client-side-navigation attribute and if so,
prevent the default event behavior in order to fetch the content of
the html, parse it, obtain the body and replace the current body.

I know that the head tags could be different but for now this is more than enough
in this blog (in the following article we will explore how to finish the idea)

But another important ting is that this client side links, have an optional optimization that
is if you just hovered your mouse with the intention of clicking, then the data fetching
already happens. Also another optional optimization is that if then link is visible in the viewport,
the data fetching is already happening.

We are gonna implement the first optimization only if the anchor
with the data atribute clientSideNavigation has the value of "hover" instead of "click"

First lets create the code that will fetch and parse the data

```ts
async function getClienPageAheadOfTime(url: URL | string, parser:DOMParser ){
    const response = await fetch(url);
    const rawHTML = await response.text();

    const { head: fetchedHead, body: fetchedBody } = parser
        .parseFromString(rawHTML, "text/html")

    const extraStyles = fetchedHead.querySelectorAll('style')

 

    return {
        appendExtraStyles(targetDocument = document){
            extraStyles.forEach((value)=>{
                    targetDocument.head.appendChild(value)
            })
        },
        replaceBody(targetDocument = document){
            targetDocument.body = fetchedBody
        }
    }

}

```

This code is generic enough to be reused inside either the click listener or the
hover

```ts
let currentCleanUp;

const { body } = document

const parser = new DOMParser();

body.addEventListener('mouseover', async (event)=>{
    const { currentTarget } = event
    const isAnchor = currentTarget instanceof HTMLAnchorElement
    if(!isAnchor)
        return;

     const { clientSideNavigation } = currentTarget.dataset

    if(clientSideNavigation !== 'hover')
        return;

    const methods = await getClienPageAheadOfTime(currentTarget.href, parser)
    // This adds extra methods to the current target to reuse in the click event
    Object.assign(currentTarget, methods)
})

body.addEventListener('click', async (event)=>{
    const { currentTarget } = event
    const isAnchor = currentTarget instanceof HTMLAnchorElement
    if(!isAnchor)
        return;

    const { clientNavigation } = currentTarget.dataset

    if(clientNavigation !== 'click')
        return;

    event.preventDefault();
    

    
    const { 
        appendExtraStyles, 
        replaceBody 
    } = (currentTarget as any).hasExtraMethods 
        ? (currentTarget as unknown as Awaited<ReturnType<typeof getClienPageAheadOfTime>> )
        : await getClienPageAheadOfTime(currentTarget.href, parser)

           // Update browser URL and navigation history
    if (typeof currentTarget.href === 'string') {
        history.pushState(null, '', currentTarget.href);
    }

    if(typeof currentCleanUp === 'function')
        currentCleanUp();

    currentCleanUp = appendExtraStyles()
    replaceBody()

 
})


```
