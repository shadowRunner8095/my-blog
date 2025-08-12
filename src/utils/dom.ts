export interface ScriptAttributes extends Partial<HTMLScriptElement> {
    src: string;
}

export function addScriptToHead(attributes: ScriptAttributes): Promise<void> {
    return new Promise((resolve, reject) => {
        const script = document.createElement('script');
        Object.assign(script, attributes);

        script.onload = () => resolve();
        script.onerror = () => reject(new Error(`Failed to load script with src: ${attributes.src}`));

        document.head.appendChild(script);
    });
}
