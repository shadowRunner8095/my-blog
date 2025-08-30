# Fetching Data in Loaders

When I want to fetch data in a SPA with a client router (like TanStack Router + React 19), I usually avoid adding extra libraries. 
Most router systems already have a **loader**, which is enough in many cases.

## How Client Routers Work

Generally speaking:

1. The root component loads.  
2. The router matches the current path to the corresponding route component.  
3. The UI shows a loading page (Suspense fallback).  
4. The loader (client loader) executes.  
5. The result of the loader is stored in a context.  
6. The matched route component renders.  

Plugins often help with wiring things together, like the ones available for TanStack Router that auto-generate the glue code.

The key steps are **4 and 5**. Usually, loaders return serializable data. But since we’re on the client, we can also return **promises** if needed.

Let’s look at some examples using **TanStack Router**. You can find [the code here](https://github.com/shadowRunner8095/experiments)

The project look something like this for reference

```text
src/
    utils/
      index.ts
    routes/
      index.lazy.tsx
      index.tsx
      root.tsx
    index.tsx

```
### Example: Loader Returning Data Directly
```tsx
// src/routes/index.tsx
export const Route = createFileRoute('/')({
  pendingComponent: () => "Loading",
  async loader() {
    const result = await fakeFetch({
      data: {
        hello: 'world',
      },
    });

    return result;
  },
});
```

Then we can use this loader data inside any component rendered by the route, like in `index.lazy.tsx`:
```tsx
// src/routes/index.lazy.tsx
import { Route as IndexRoute } from '.'

export const Route = createLazyFileRoute('/')({
  component: () => {
    const { hello } = IndexRoute.useData();
    return <div>{hello}</div>
  },
});

```

The catch: everything in the page will suspend until the loader resolves, so the **pendingComponent** is shown (say, for ~1500ms).  
This isn’t always bad, but what if your client wants the **page visible immediately** and only show a loading state for one small section?

```tsx

// src/routes/index.tsx
export const Route = createFileRoute('/')({
  pendingComponent: () => "Loading",
  loader() {
    return fakeFetch({
      data: {
        hello: 'world',
      },
    });
  },
});

```

Here the loader returns a **promise**. You can then consume it with React’s `use` hook:
```tsx
import { Route as IndexRoute } from '.'

const ShowData = () => {
  const promiseData = IndexRoute.useData();
  const { hello } = use(promiseData);

  return <div>{hello}</div>
}

export const Route = createLazyFileRoute('/')({
  component: () => {
    return (
      <div>
        <p>This does not suspend</p>
        <Suspense fallback={<div>Here should be a skeleton loader</div>}>
          <ShowData />
        </Suspense>
      </div>
    )
  },
});

```

The trick: the **Suspense fallback** is now just a local pending state, not blocking the entire page.


### Reusing the Same Promise

What if you need the same data elsewhere on the page? No problem. Since the promise reference is the same, if it’s already resolved, no new fetch will trigger.

```tsx
import { Route as IndexRoute } from '.'

const ShowData = () => {
  const promiseData = IndexRoute.useData();
  const { hello } = use(promiseData);

  return <div>{hello}</div>
}

const ShowDataOrLoading = () => (
  <Suspense fallback={<div>Here should be a skeleton loader</div>}>
    <ShowData />
  </Suspense>
)

const ShowOnClick = () => {
  const [isVisible, setIsVisible] = useState(false);
  const onClick = useCallback(() => setIsVisible(true), []);

  return (
    <div>
      <button onClick={onClick}>Show</button>
      {isVisible && <ShowDataOrLoading />}
    </div>
  )
}

export const Route = createLazyFileRoute('/')({
  component: () => {
    return (
      <div>
        <p>This does not suspend</p>
        <ShowDataOrLoading />
        <ShowOnClick />
      </div>
    )
  },
});

```

When you click the button, no loading UI shows up — the promise is already resolved, so React skips the suspense fallback.


### Passing Data Between Pages

In an SPA, you can often pass data between pages via **navigation hooks**, without needing a global store.


### What About Forms?

For form submissions, you can use **form actions** and `useFormStatus` to handle async results cleanly.


### Conditional Fetching

Sometimes you don’t want to resolve a promise immediately (e.g., waiting for user input).  
The key is to **preserve the same promise reference** so your component doesn’t get stuck in an infinite suspend/resume loop.  

One practical example: building an infinite scroll search app — where new data is only fetched once the user types or scrolls.

I know we should throttle the requests, but for now let’s skip that to keep things simple.

```tsx
export function MainSearch({ search }) {
  const [searchPromise, setSearchPromise] = useState(null)

  const onChange = useCallback((event) => {
    const { value } = event.target
    setSearchPromise(search(value))
  }, [search])

  return (
    <div>
      <input />
      <Suspense fallback={"loading"}>
        <Contents data={searchPromise} />
      </Suspense>
    </div>
  )
}

function Contents({ data }) {
  const result = use(data)

  return (
    <div>
      {result.map((props) => <CharacterCard key={props.id} {...props} />)}
    </div>
  )
}
```

But now we have a problem: this only gives us a single set of results.  
If we want **infinite scrolling** or **pagination**, we need something more.  

I’ll go with infinite scrolling for this example. Be aware that I won’t optimize with **windowing** yet — we’ll just focus on the functionality.  

The key issue is that the component suspends and remounts because the promise reference changes.  
So instead of a single promise, we need to think in terms of **an array of promises**.  

We can then use `IntersectionObserver` + `useEffect` to trigger loading more pages when the sentinel enters the viewport.  
(Later we could refactor this into a `useSyncExternalStore`, but for now let’s keep it simple.)


```tsx

export function MainSearch({ search }) {
  const sentinelRef = useRef()
  const paginationRef = useRef()

  const [searchPromises, setSearchPromises] = useState(null)

  const onChange = useCallback((event) => {
    const { value } = event.target
    setSearchPromises([search(value)])
  }, [search])

  useEffect(() => {
    paginationRef.current = { page: 0 }

    const observer = new IntersectionObserver((entries) => {
      if (entries[0].isIntersecting) {
        setSearchPromises(prev =>
          [...prev, search(value, ++paginationRef.current.page)]
        )
      }
    })

    if (sentinelRef.current) observer.observe(sentinelRef.current)
    return () => observer.disconnect()
  }, [])

  return (
    <div>
      <input />
      <div>
        {searchPromises.map((promise, index) => (
          <Suspense key={`batch-${index}`} fallback={"loading"}>
            <Contents data={promise} />
            {index + 1 === searchPromises.length && <div ref={sentinelRef}></div>}
          </Suspense>
        ))}
      </div>
    </div>
  )
}


```

Nice! Now the UI reacts to **what actually changes**: the array of promises.  
The sentinel for infinite scroll only renders on the last item, so we avoid keeping extra state like *isFetchingNext* or *isPending*.  
Suspense does that work for us.  

But backend errors can happen, so let’s add an **ErrorBoundary**.

```bash
pnpm add react-error-boundary
```

(Just reminding you that react error boundary error does not come inside react package).

```tsx
import { useState, useCallback, useRef, useEffect, Suspense, use, memo } from 'react';
import { CharacterCard, IntersectionSentinel, LoadingSpinner, SearchInput } from './SearchComponents';
import { ErrorBoundary, type FallbackProps } from 'react-error-boundary';

interface SearchFunction {
  (value: string, page?: number): Promise<Array<{ id: string; text: string }>>;
}

interface MainSearchProps {
  search: SearchFunction;
}

interface ContentsProps {
  data: Promise<Array<{ id: string; text: string }>>;
}

function Contents({ data }: ContentsProps) {
  const result = use(data);
  console.log('rendering', data)

  return (
    <div>
      {result?.map((props) => <CharacterCard key={props.id} {...props} />)}
    </div>
  );
}

const MemoContents = memo(Contents);

const CustomError = ({ error }: FallbackProps) => {
  return <div style={{ minHeight: '100vh' }}>
    {error.message}
  </div>
}

export function MainSearch({ search }: MainSearchProps) {
  const paginationRef = useRef<{ page: 0 }>(null);
  const currentValueRef = useRef<string>('');

  const [searchPromises, setSearchPromises] = useState<Array<Promise<Array<{ id: string; text: string }>>>>([]);

  const onChange = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    const { value } = event.target;
    currentValueRef.current = value;

    if (value.trim()) {
      paginationRef.current = { page: 0 };
      setSearchPromises([search(value, 0)]);
    } else {
      setSearchPromises([]);
    }
  }, [search]);

  const onBottom = useCallback(() => {
    if (!paginationRef.current)
      paginationRef.current = { page: 0 };

    setSearchPromises(prev =>
      [...prev, search(currentValueRef.current, ++paginationRef.current.page)]
    );
  }, [search]);

  return (
    <div>
      <SearchInput onChange={onChange} />
      <ErrorBoundary resetKeys={[currentValueRef.current]} FallbackComponent={CustomError}>
        <div>
          {searchPromises.map((promise, index) => (
            <Suspense
              key={`batch-${index}`}
              fallback={<LoadingSpinner />}
            >
              <MemoContents data={promise} />
              {index + 1 === searchPromises.length && (
                <IntersectionSentinel onIntersect={onBottom} />
              )}
            </Suspense>
          ))}
        </div>
      </ErrorBoundary>
    </div>
  );
}
```

Now the error boundary resets every time the input changes, and shows an error message if something goes wrong.

Something similar could be built with **React Query**, and let me be clear:  
I *love* React Query. I’m not saying this approach is “better.” React Query’s API is ergonomic, well-crafted, and has a good balance of trade-offs.  

This approach with Suspense just explores another angle — it has its own caveats, trade-offs, and ergonomics.  

The important part: React Query has a powerful **in-memory cache** out of the box.  
This Suspense approach is closer to cases where caching isn’t needed.  

As I mentioned before, if you manage your promise references carefully, you don’t need a global store to reuse data.  
But ergonomics matter: React Query makes this easier, while a “manual Suspense” approach may suit **medium-sized apps** or specialized cases.

The same principle could be applied to **vanilla JS** as well. We’re just leaning on React primitives instead of fighting them.  

Of course, abstractions could be built around these ideas — but that’s an exercise for later.  

Remember that there are **cases where Suspense is *not* the right fit**, for example, when you need to react to errors inside an input, update styles, or set `aria` props dynamically.
