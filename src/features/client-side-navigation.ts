import { clientNavigation } from '../../features.json';

async function getClienPageAheadOfTime(
    url: URL | string,
    parser:DOMParser,
){
    const response = await fetch(url);
    const rawHTML = await response.text();

    const { head: fetchedHead, body: fetchedBody } = parser
        .parseFromString(rawHTML, "text/html")

    const extraStyles = fetchedHead.querySelectorAll('style')

    //TODO:  herre I can add to the new document head the needed scripts and css?

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

// execute to activate client side navigation
const appendListeners = ()=>{
    let currentCleanUp;

    const parser = new DOMParser();

    document.addEventListener('mouseover', async (event)=>{
        const { target } = event
        const isAnchor = target instanceof HTMLAnchorElement

        if(!isAnchor)
            return;
        if((target as any).hasExtraMethods)
            return;
         const { clientNavigation } = target.dataset

        if(clientNavigation !== 'hover')
            return;

        const methods = await getClienPageAheadOfTime(target.href, parser)

        Object.assign(target, methods)
    })

    document.addEventListener('click', async (event)=>{
        const { target } = event
        const isAnchor = target instanceof HTMLAnchorElement

        if(!isAnchor)
            return;

        const { clientNavigation } = target.dataset

        if((clientNavigation !== 'click') && (clientNavigation !== 'hover'))
            return;

        event.preventDefault();

        const {
            appendExtraStyles,
            replaceBody
        } = (target as any).hasExtraMethods
            ? (target as unknown as Awaited<ReturnType<typeof getClienPageAheadOfTime>>)
            : await getClienPageAheadOfTime(target.href, parser)


        if (typeof currentCleanUp === 'function')
            currentCleanUp();

        currentCleanUp = appendExtraStyles();
        replaceBody();
        // Update the URL and history
        history.pushState(null, '', target.href);
    })
}

export function initializeClientSideNavigation() {
    if(clientNavigation) {
        appendListeners()
    }
}
