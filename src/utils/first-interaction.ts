import { StatefulObserver } from './observer';

// Use the new stateful observer to hold the interaction state, initialized to false.
const interactionObserver = new StatefulObserver<boolean>(false);

const events: (keyof WindowEventMap)[] = ['click', 'scroll', 'keydown', 'mousemove', 'touchstart'];

// This function will be used as the event listener.
const trigger = () => {
    // Notify the observer that an interaction has occurred. This will update the state to true.
    interactionObserver.notify(true);

    // Manually remove all event listeners after the first trigger, as requested.
    events.forEach(event => {
        window.removeEventListener(event, trigger);
    });
};

// Only add the event listeners if the interaction has not already happened.
if (interactionObserver.getState() === false) {
    events.forEach(event => {
        // Note: We are not using { once: true } here to implement manual removal.
        window.addEventListener(event, trigger, { passive: true });
    });
}

/**
 * Subscribes a callback to be executed on the first user interaction.
 * If the interaction has already occurred, the callback is executed immediately.
 */
export function onFirstInteraction(callback: () => void) {
    const unsubscribe = interactionObserver.subscribe(hasInteracted => {
        if (hasInteracted) {
            callback();
            // After the callback is fired once, we unsubscribe it to prevent future executions.
            unsubscribe();
        }
    });
}
