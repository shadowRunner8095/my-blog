async function getClienPageAheadOfTime(
    url: URL | string, 
    parser:DOMParser,
){
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

            return ()=>{
                extraStyles.forEach(value=>targetDocument.head.removeChild(value))
            }
        },
        replaceBody(targetDocument = document){
            targetDocument.body = fetchedBody
        },
        hasExtraMethods: true
    }

}

let currentCleanUp;

const { body } = document

const parser = new DOMParser();

body.addEventListener('click', async (event)=>{
    const { currentTarget } = event
    const isAnchor = currentTarget instanceof HTMLAnchorElement
    if(!isAnchor)
        return;

    const { clientSideNavigation } = currentTarget.dataset

    if(clientSideNavigation !== 'click')
        return;

    event.preventDefault();

    const {
        appendExtraStyles,
        replaceBody
    } = (currentTarget as any).hasExtraMethods
        ? (currentTarget as unknown as Awaited<ReturnType<typeof getClienPageAheadOfTime>>)
        : await getClienPageAheadOfTime(currentTarget.href, parser)

    // Update browser URL and navigation history
    if (typeof currentTarget.href === 'string') {
        history.pushState(null, '', currentTarget.href);
    }

    if (typeof currentCleanUp === 'function')
        currentCleanUp();

    currentCleanUp = appendExtraStyles();
    replaceBody();
})

body.addEventListener('mouseover', async (event)=>{
    const { currentTarget } = event
    const isAnchor = currentTarget instanceof HTMLAnchorElement
    if(!isAnchor)
        return;

     const { clientSideNavigation } = currentTarget.dataset

    if(clientSideNavigation !== 'hover')
        return;

    const methods = await getClienPageAheadOfTime(currentTarget.href, parser)

    Object.assign(currentTarget, methods)
})