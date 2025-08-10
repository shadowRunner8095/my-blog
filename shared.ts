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

body.addEventListener('mouseover', async (event)=>{
    const { target } = event
    const isAnchor = target instanceof HTMLAnchorElement
 
    if(!isAnchor)
        return;

     const { clientNavigation } = target.dataset

    if(clientNavigation !== 'hover')
        return;

    const methods = await getClienPageAheadOfTime(target.href, parser)

    Object.assign(target, methods)
})

body.addEventListener('click', async (event)=>{
    const { target } = event
    const isAnchor = target instanceof HTMLAnchorElement
  console.log(isAnchor, target)
    if(!isAnchor)
        return;

    const { clientNavigation } = target.dataset
    console.log(clientNavigation)
    if((clientNavigation !== 'click') ||(clientNavigation !== 'hover'))
        return;
    console.log('azucar')
    event.preventDefault();

    const {
        appendExtraStyles,
        replaceBody
    } = (target as any).hasExtraMethods
        ? (target as unknown as Awaited<ReturnType<typeof getClienPageAheadOfTime>>)
        : await getClienPageAheadOfTime(target.href, parser)

    // Update browser URL and navigation history
    if (typeof target.href === 'string') {
        history.pushState(null, '', target.href);
    }

    if (typeof currentCleanUp === 'function')
        currentCleanUp();

    currentCleanUp = appendExtraStyles();
    replaceBody();
})

