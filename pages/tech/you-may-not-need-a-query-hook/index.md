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

The catch: everything in the page will suspend until the loader resolves, so the **pendingComponent** is shown (say, for ~1500ms). This isn’t always bad, but what if your client wants the **page visible immediately** and only show a loading state for one small section?

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
        <Suspense
          fallback={
            <div>
              Here should be a skeleton loader
            </div>
         }
        >
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
  <Suspense
    fallback={
      <div>
        Here should be a skeleton loader
      </div>
    }
  >
    <ShowData />
  </Suspense>
)

const ShowOnClick = () => {
  const [isVisible, setIsVisible] = useState(false);
  const onClick = useCallback(() => setIsVisible(true), []);

  return (
    <div>
      <button onClick={onClick}>
        Show
      </button>
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

When you click the button, no loading UI shows up ,  the promise is already resolved, so React skips the suspense fallback.


### Passing Data Between Pages

In an SPA, you can often pass data between pages via **navigation hooks**, without needing a global store.


### What About Forms?

For form submissions, you can use **form actions** and `useFormStatus` to handle async results cleanly.


### Conditional Fetching

Sometimes you don’t want to resolve a promise immediately (e.g., waiting for user input). The key is to **preserve the same promise reference** so your component doesn’t get stuck in an infinite suspend/resume loop.  

One practical example: building an infinite scroll search app where new data is only fetched once the user types or scrolls.

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
      {result.map((props) => <CharacterCard
        key={props.id}
        {...props}
      />)
      }
    </div>
  )
}
```

But now we have a problem: this only gives us a single set of results. If we want **infinite scrolling** or **pagination**, we need something more.  

I’ll go with infinite scrolling for this example. Be aware that I won’t optimize with **windowing** yet ,  we’ll just focus on the functionality.  

The key issue is that the component suspends and remounts because the promise reference changes. So instead of a single promise, we need to think in terms of **an array of promises**.  

We can then use `IntersectionObserver` + `useEffect` to trigger loading more pages when the sentinel enters the viewport. (Later we could refactor this into a `useSyncExternalStore`, but for now let’s keep it simple.)


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
          [
            ...prev,
            search(value, ++paginationRef.current.page)
          ]
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
            {index + 1 === searchPromises.length
              && <div ref={sentinelRef}></div>
            }
          </Suspense>
        ))}
      </div>
    </div>
  )
}


```

Nice! Now the UI reacts to **what actually changes**: the array of promises.  

The sentinel for infinite scroll only renders on the last item, so we avoid keeping extra state like *isFetchingNext* or *isPending*. Suspense does that work for us.  

But backend errors can happen, so let’s add an **ErrorBoundary**.

```bash
pnpm add react-error-boundary
```

(Just reminding you that react error boundary error does not come inside react package).

```tsx
import {
  useState,
  useCallback,
  useRef,
  useEffect,
  Suspense,
  use,
  memo,
  type ChangeEvent
} from 'react';
import {
  CharacterCard,
  IntersectionSentinel,
  LoadingSpinner,
  SearchInput
} from './SearchComponents';
import {
  ErrorBoundary,
  type FallbackProps
} from 'react-error-boundary';

interface Result {
  id: string;
  text: string;
}

interface SearchFunction {
  (
    value: string,
    page?: number
  ): Promise<Result[]>;
}

interface MainSearchProps {
  search: SearchFunction;
}

interface ContentsProps {
  data: Promise<Result[]>;
}

function Contents({ data }: ContentsProps) {
  const result = use(data);

  return (
    <div>
      {result?.map((props) =>
        <CharacterCard
          key={props.id}
          {...props} />
        )
      }
    </div>
  );
}

const MemoContents = memo(Contents);

const CustomError = ({
  error
}: FallbackProps) => {
  return <div
    style={{
      minHeight: '100vh'
    }}
  >
    {error.message}
  </div>
}

export function MainSearch(
  {
    search
  }: MainSearchProps
) {
  const paginationRef = useRef<{ page: number }>(null);
  const currentValueRef = useRef<string>('');

  const [searchPromises, setSearchPromises] = useState<
    Array<Promise<Array<Result>>>
  >([]);

  const onChange = useCallback((
    event: ChangeEvent<HTMLInputElement>
  ) => {
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
      [
        ...prev,
        search(
          currentValueRef.current,
          ++paginationRef.current.page
        )
      ]
    );
  }, [search]);

  return (
    <div>
      <SearchInput
        onChange={onChange}
      />
      <ErrorBoundary
        resetKeys={[currentValueRef.current]}
        FallbackComponent={CustomError}
      >
        <div>
          {searchPromises
            .map((promise, index) => (
              <Suspense
                key={`batch-${index}`}
                fallback={<LoadingSpinner />}
              >
                <MemoContents data={promise} />
                {index + 1 === searchPromises.length && (
                  <IntersectionSentinel
                    onIntersect={onBottom}
                  />
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

Something similar could be built with **React Query**, and let me be clear: I *love* React Query. I’m not saying this approach is “better.” React Query’s API is ergonomic, well-crafted, and has a good balance of trade-offs.  

This approach with Suspense just explores another angle, it has its own caveats, trade-offs, and ergonomics.  

The important part: React Query has a powerful **in-memory cache** out of the box. This Suspense approach is closer to cases where caching isn’t needed.  

As I mentioned before, if you manage your promise references carefully, you don’t need a global store to reuse data. React Query have another aproach, having hooks that explicity return the state of the query and the resolved data, this makes sense in some scenarios aand so the "Suspense” approach may suit **medium-sized apps** or specialized cases.

The same principle could be applied to **vanilla JS** as well. We’re just leaning on React primitives instead of fighting them.  

Of course, abstractions could be built around these ideas, but that’s an exercise for later.  

Remember that there are **cases where Suspense is *not* the right fit**, for example, when you need to react to errors inside an input, update styles, or set `aria` props dynamically.

## Nevertheless, React 19 still has ways to do this without extra libraries

Let’s explore a case: An input could be valid in format but invalid when checked against an async operation, which returns errors.

We need to wait for the async operation to resolve before deciding if something is valid. Then we render, next to the input, an “ok” or “wrong” icon and show the error text below the input. Additionally, a wrapper around the input and the icon should change style when something is invalid, like turning its border red.

If we keep saving a promise in state, we hit a problem: only the icon and error box can suspend with a loading fallback. But the wrapper also needs the awaited result to decide what border color to show. Since the input is its child, if the wrapper suspends, the input will lose its state, or worse, disappear and show the nearest Suspense fallback in the tree. That’s visually nasty, and we lose whatever the user was typing, because the component remounts after suspending.

So... is there any way to not fight the framework?  
Well yes: `useDeferredValue` integrates with Suspense so React doesn’t replace the UI of parts using the deferred value, even when that value should trigger a Suspense state visually.

Let’s create the input. **But we must separate things logically.** The components that need the resolved data from the promise are:  
- `InputLayout`  
- `IconWrapper`  
- `ErrorText`

Only the last two need to suspend with fallbacks. Let’s start by creating the layout:

```tsx
interface InputValidationResponse {
  error?: Error;
  data?: any;
}

interface InputLayoutProps {
  deferedValue: Promise<InputValidationResponse>;
  isStale: boolean;
}

export function IncorrectInputLayout({
  deferedValue,
 ...props
}: PropsWithChildren<InputLayoutProps>){
  const data = use(deferedValue)

  return <div
    style={{
      borderStyle: 'solid',
      borderWidth: '1px',
      borderColor: data.error ? 'red' : 'black',
      display: 'flex'
    }}
    {...props}
  />
}
```
```tsx
interface InputValidationResponse {
  error?: Error;
  data?: any;
}

interface InputLayoutProps {
  deferedValue: Promise<InputValidationResponse>;
  isStale: boolean;
}

export function IncorrectInputLayout({deferedValue, ...props}: PropsWithChildren<InputLayoutProps>){
  const data = use(deferedValue)

  return <div
    style={{
      borderStyle: 'solid',
      borderWidth: '1px',
      borderColor: data.error ? 'red' : 'black',
      display: 'flex'
    }}
    {...props}
  />
}

```

As we can see, this is agnostic, it can be reused without knowing if it’s tied to Suspense or `useDeferredValue`. Now let’s continue with the other two components:

```tsx
// Very bad icon example, lol
function IconWrapper(
  {
    query
  }: {
    query: Promise<InputValidationResponse>
  }
) {
  const { error } = use(query)
  return error ? "Bad" : "Ok"
}

// Again, a bad error example, but just for demo
function ErrorText(
  {
    query
  }: {
    query: Promise<InputValidationResponse>
  }) {
  const { error } = use(query)
  if (error) return error.message
}

```
Let’s try creating code that will fail:

```tsx
interface SearchInputWithAsyncValidation {
  validate(value: string): Promise<InputValidationResponse>;
  InputLayoutComponent?: FC<PropsWithChildren<InputLayoutProps>>;
}

const defaultValue = Promise.resolve({})

export function SearchInputWithPromiseValidation(
  {
    validate,
    InputLayoutComponent = IncorrectInputLayout
  }: SearchInputWithAsyncValidation
) {
  const [query, setQuery] = useState<
    Promise<InputValidationResponse>
  >(defaultValue)

  const onChange = (
    event: ChangeEvent<HTMLInputElement>
  ) => {
    setQuery(validate(event.target.value))
  }

  return <Suspense fallback={'Outer Loading'}>
    <div>
      <InputLayoutComponent
        deferedValue={query}
      >
        <input
          style={{ border: 'none', outline: 'none' }}
          onChange={onChange}
        />
        <Suspense fallback={<div>Loading</div>}>
          <IconWrapper query={query} />
        </Suspense>
      </InputLayoutComponent>
      <Suspense>
        <ErrorText query={query} />
      </Suspense>
    </div>
  </Suspense>
}

```

In this case, the input disappears and is replaced by the outer Suspense, remounting everything. This is a common pitfall. But no fear, here comes the deferred value.

The final glue code looks like this:
```tsx
function InputLayout({
  deferedValue,
  ...props
}: PropsWithChildren<InputLayoutProps>) {
  const promise = useDeferredValue(deferedValue)
  const data = use(promise)

  return <div
    style={{
      borderStyle: 'solid',
      borderWidth: '1px',
      borderColor: data.error ? 'red' : 'black',
      display: 'flex'
    }}
    {...props}
  />
}

```
Now, using this `InputLayout` instead of the incorrect one, the outer Suspense no longer takes over.

I know inline `style` props aren’t ideal, but remember, this is just an example. The focus is the behavior, not styling.

Now the input doesn’t disappear, because as we said, using the deferred value avoids re-rendering the subscribed parts.

Error boundaries are missing, but honestly I prefer to include errors in the final data instead of throwing. That way, when a *real* error happens, it’s not some “expected” error like a 404, but an actual app error we can catch with a Boundary. A 404 isn’t an “error” from my perspective.

Of course, there are many other ways to implement this pattern, like having promises injected from context that manages references or caches, allowing us to build powerful reactive systems. Another option is `useSyncExternalStore`, which can return different promises if needed. (This means you could use Zustand to return promises instead of just storing data, though that requires a different mental model than global stores. And since it’s more for sync operations, chances are low you’d use it here, but who knows.)

Now, we can lift the defered value also if we do not want to include it fro the input layout if it is not its responsability for
the design, so as you can see we have several ways to use fluent React 19.
