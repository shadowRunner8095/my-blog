import { onFirstInteraction } from '../utils/first-interaction';
import { addScriptToHead } from '../utils/dom';

export function setupMermaid() {
    // Find all elements that are designated for Mermaid rendering.
    const mermaidElements = document.querySelectorAll('.language-mermaid');

    // If there are no Mermaid elements on the page, there's nothing to do.
    if (mermaidElements.length === 0) {
        return;
    }

    // Subscribe to the first user interaction event.
    onFirstInteraction(async () => {
        // When the user interacts, prepare the elements for Mermaid.
        mermaidElements.forEach(element => {
            element.classList.add('mermaid');
        });

        try {
            // Add the Mermaid.js script to the document head and wait for it to load.
            await addScriptToHead({
                src: 'https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js',
                async: true,
            });
        } catch (error) {
            console.error('Failed to load Mermaid.js script:', error);
        }
    });
}
