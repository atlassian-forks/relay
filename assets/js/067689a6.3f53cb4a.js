(self.webpackChunk=self.webpackChunk||[]).push([[41707],{3905:(e,n,t)=>{"use strict";t.d(n,{Zo:()=>c,kt:()=>m});var a=t(67294);function r(e,n,t){return n in e?Object.defineProperty(e,n,{value:t,enumerable:!0,configurable:!0,writable:!0}):e[n]=t,e}function i(e,n){var t=Object.keys(e);if(Object.getOwnPropertySymbols){var a=Object.getOwnPropertySymbols(e);n&&(a=a.filter((function(n){return Object.getOwnPropertyDescriptor(e,n).enumerable}))),t.push.apply(t,a)}return t}function o(e){for(var n=1;n<arguments.length;n++){var t=null!=arguments[n]?arguments[n]:{};n%2?i(Object(t),!0).forEach((function(n){r(e,n,t[n])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(t)):i(Object(t)).forEach((function(n){Object.defineProperty(e,n,Object.getOwnPropertyDescriptor(t,n))}))}return e}function s(e,n){if(null==e)return{};var t,a,r=function(e,n){if(null==e)return{};var t,a,r={},i=Object.keys(e);for(a=0;a<i.length;a++)t=i[a],n.indexOf(t)>=0||(r[t]=e[t]);return r}(e,n);if(Object.getOwnPropertySymbols){var i=Object.getOwnPropertySymbols(e);for(a=0;a<i.length;a++)t=i[a],n.indexOf(t)>=0||Object.prototype.propertyIsEnumerable.call(e,t)&&(r[t]=e[t])}return r}var l=a.createContext({}),d=function(e){var n=a.useContext(l),t=n;return e&&(t="function"==typeof e?e(n):o(o({},n),e)),t},c=function(e){var n=d(e.components);return a.createElement(l.Provider,{value:n},e.children)},p={inlineCode:"code",wrapper:function(e){var n=e.children;return a.createElement(a.Fragment,{},n)}},u=a.forwardRef((function(e,n){var t=e.components,r=e.mdxType,i=e.originalType,l=e.parentName,c=s(e,["components","mdxType","originalType","parentName"]),u=d(t),m=r,h=u["".concat(l,".").concat(m)]||u[m]||p[m]||i;return t?a.createElement(h,o(o({ref:n},c),{},{components:t})):a.createElement(h,o({ref:n},c))}));function m(e,n){var t=arguments,r=n&&n.mdxType;if("string"==typeof e||r){var i=t.length,o=new Array(i);o[0]=u;var s={};for(var l in n)hasOwnProperty.call(n,l)&&(s[l]=n[l]);s.originalType=e,s.mdxType="string"==typeof e?e:r,o[1]=s;for(var d=2;d<i;d++)o[d]=t[d];return a.createElement.apply(null,o)}return a.createElement.apply(null,t)}u.displayName="MDXCreateElement"},68629:(e,n,t)=>{"use strict";t.d(n,{Z:()=>p});var a=t(44256),r=t(67294);function i(){var e=window.encodeURI(JSON.stringify({title:"Feedback about "+window.location.pathname,description:"**!!! Required !!!**\n\nPlease modify the task description to let us know how the docs can be improved.\n\n**Please do not ask support questions via this form! Instead, ask in fburl.com/relay_support**",tag_ids:{add:[0xac96423e5b680,0x64079768ac750]}}));window.open("https://www.internalfb.com/tasks/?n="+e)}function o(e){var n=e.children;return r.createElement("div",{className:"docsRating",id:"docsRating"},r.createElement("hr",null),n)}var s=function(){var e=r.useState(!1),n=e[0],t=e[1],a=function(e){t(!0),function(e){window.ga&&window.ga("send",{hitType:"event",eventCategory:"button",eventAction:"feedback",eventValue:e})}(e)};return n?"Thank you for letting us know!":r.createElement(r.Fragment,null,"Is this page useful?",r.createElement("svg",{className:"i_thumbsup",alt:"Like",id:"docsRating-like",xmlns:"http://www.w3.org/2000/svg",viewBox:"0 0 81.13 89.76",onClick:function(){return a(1)}},r.createElement("path",{d:"M22.9 6a18.57 18.57 0 002.67 8.4 25.72 25.72 0 008.65 7.66c3.86 2 8.67 7.13 13.51 11 3.86 3.11 8.57 7.11 11.54 8.45s13.59.26 14.64 1.17c1.88 1.63 1.55 9-.11 15.25-1.61 5.86-5.96 10.55-6.48 16.86-.4 4.83-2.7 4.88-10.93 4.88h-1.35c-3.82 0-8.24 2.93-12.92 3.62a68 68 0 01-9.73.5c-3.57 0-7.86-.08-13.25-.08-3.56 0-4.71-1.83-4.71-4.48h8.42a3.51 3.51 0 000-7H12.28a2.89 2.89 0 01-2.88-2.88 1.91 1.91 0 01.77-1.78h16.46a3.51 3.51 0 000-7H12.29c-3.21 0-4.84-1.83-4.84-4a6.41 6.41 0 011.17-3.78h19.06a3.5 3.5 0 100-7H9.75A3.51 3.51 0 016 42.27a3.45 3.45 0 013.75-3.48h13.11c5.61 0 7.71-3 5.71-5.52-4.43-4.74-10.84-12.62-11-18.71-.15-6.51 2.6-7.83 5.36-8.56m0-6a6.18 6.18 0 00-1.53.2c-6.69 1.77-10 6.65-9.82 14.5.08 5.09 2.99 11.18 8.52 18.09H9.74a9.52 9.52 0 00-6.23 16.9 12.52 12.52 0 00-2.07 6.84 9.64 9.64 0 003.65 7.7 7.85 7.85 0 00-1.7 5.13 8.9 8.9 0 005.3 8.13 6 6 0 00-.26 1.76c0 6.37 4.2 10.48 10.71 10.48h13.25a73.75 73.75 0 0010.6-.56 35.89 35.89 0 007.58-2.18 17.83 17.83 0 014.48-1.34h1.35c4.69 0 7.79 0 10.5-1 3.85-1.44 6-4.59 6.41-9.38.2-2.46 1.42-4.85 2.84-7.62a41.3 41.3 0 003.42-8.13 48 48 0 001.59-10.79c.1-5.13-1-8.48-3.35-10.55-2.16-1.87-4.64-1.87-9.6-1.88a46.86 46.86 0 01-6.64-.29c-1.92-.94-5.72-4-8.51-6.3l-1.58-1.28c-1.6-1.3-3.27-2.79-4.87-4.23-3.33-3-6.47-5.79-9.61-7.45a20.2 20.2 0 01-6.43-5.53 12.44 12.44 0 01-1.72-5.36 6 6 0 00-6-5.86z"})),r.createElement("svg",{className:"i_thumbsdown",alt:"Dislike",id:"docsRating-dislike",xmlns:"http://www.w3.org/2000/svg",viewBox:"0 0 81.13 89.76",onClick:function(){return a(0)}},r.createElement("path",{d:"M22.9 6a18.57 18.57 0 002.67 8.4 25.72 25.72 0 008.65 7.66c3.86 2 8.67 7.13 13.51 11 3.86 3.11 8.57 7.11 11.54 8.45s13.59.26 14.64 1.17c1.88 1.63 1.55 9-.11 15.25-1.61 5.86-5.96 10.55-6.48 16.86-.4 4.83-2.7 4.88-10.93 4.88h-1.35c-3.82 0-8.24 2.93-12.92 3.62a68 68 0 01-9.73.5c-3.57 0-7.86-.08-13.25-.08-3.56 0-4.71-1.83-4.71-4.48h8.42a3.51 3.51 0 000-7H12.28a2.89 2.89 0 01-2.88-2.88 1.91 1.91 0 01.77-1.78h16.46a3.51 3.51 0 000-7H12.29c-3.21 0-4.84-1.83-4.84-4a6.41 6.41 0 011.17-3.78h19.06a3.5 3.5 0 100-7H9.75A3.51 3.51 0 016 42.27a3.45 3.45 0 013.75-3.48h13.11c5.61 0 7.71-3 5.71-5.52-4.43-4.74-10.84-12.62-11-18.71-.15-6.51 2.6-7.83 5.36-8.56m0-6a6.18 6.18 0 00-1.53.2c-6.69 1.77-10 6.65-9.82 14.5.08 5.09 2.99 11.18 8.52 18.09H9.74a9.52 9.52 0 00-6.23 16.9 12.52 12.52 0 00-2.07 6.84 9.64 9.64 0 003.65 7.7 7.85 7.85 0 00-1.7 5.13 8.9 8.9 0 005.3 8.13 6 6 0 00-.26 1.76c0 6.37 4.2 10.48 10.71 10.48h13.25a73.75 73.75 0 0010.6-.56 35.89 35.89 0 007.58-2.18 17.83 17.83 0 014.48-1.34h1.35c4.69 0 7.79 0 10.5-1 3.85-1.44 6-4.59 6.41-9.38.2-2.46 1.42-4.85 2.84-7.62a41.3 41.3 0 003.42-8.13 48 48 0 001.59-10.79c.1-5.13-1-8.48-3.35-10.55-2.16-1.87-4.64-1.87-9.6-1.88a46.86 46.86 0 01-6.64-.29c-1.92-.94-5.72-4-8.51-6.3l-1.58-1.28c-1.6-1.3-3.27-2.79-4.87-4.23-3.33-3-6.47-5.79-9.61-7.45a20.2 20.2 0 01-6.43-5.53 12.44 12.44 0 01-1.72-5.36 6 6 0 00-6-5.86z"})))},l=function(){return r.createElement("p",null,"Let us know how these docs can be improved by",r.createElement("a",{className:"button",role:"button",tabIndex:0,onClick:i},"Filing a task"))},d=function(){return r.createElement(o,null,r.createElement(l,null),r.createElement(s,null))},c=function(){return r.createElement(o,null,r.createElement(s,null))};const p=function(){return(0,a.fbContent)({internal:r.createElement(d,null),external:r.createElement(c,null)})}},13246:(e,n,t)=>{"use strict";t.r(n),t.d(n,{frontMatter:()=>l,contentTitle:()=>d,metadata:()=>c,toc:()=>p,default:()=>f});var a=t(22122),r=t(19756),i=(t(67294),t(3905)),o=t(68629),s=(t(44256),["components"]),l={id:"rendering-partially-cached-data",title:"Rendering Partially Cached Data",slug:"/guided-tour/reusing-cached-data/rendering-partially-cached-data/",description:"Relay guide to rendering partially cached data",keywords:["partially cached data","renderPolicy"]},d=void 0,c={unversionedId:"guided-tour/reusing-cached-data/rendering-partially-cached-data",id:"version-v12.0.0/guided-tour/reusing-cached-data/rendering-partially-cached-data",isDocsHomePage:!1,title:"Rendering Partially Cached Data",description:"Relay guide to rendering partially cached data",source:"@site/versioned_docs/version-v12.0.0/guided-tour/reusing-cached-data/rendering-partially-cached-data.md",sourceDirName:"guided-tour/reusing-cached-data",slug:"/guided-tour/reusing-cached-data/rendering-partially-cached-data/",permalink:"/docs/guided-tour/reusing-cached-data/rendering-partially-cached-data/",editUrl:"https://github.com/facebook/relay/tree/main/website/versioned_docs/version-v12.0.0/guided-tour/reusing-cached-data/rendering-partially-cached-data.md",version:"v12.0.0",lastUpdatedBy:"Andrey Lunyov",lastUpdatedAt:1635261416,formattedLastUpdatedAt:"10/26/2021",frontMatter:{id:"rendering-partially-cached-data",title:"Rendering Partially Cached Data",slug:"/guided-tour/reusing-cached-data/rendering-partially-cached-data/",description:"Relay guide to rendering partially cached data",keywords:["partially cached data","renderPolicy"]},sidebar:"version-v12.0.0/docs",previous:{title:"Staleness of Data",permalink:"/docs/guided-tour/reusing-cached-data/staleness-of-data/"},next:{title:"Filling in Missing Data (Missing Field Handlers)",permalink:"/docs/guided-tour/reusing-cached-data/filling-in-missing-data/"}},p=[{value:"Fragments as boundaries for partial rendering",id:"fragments-as-boundaries-for-partial-rendering",children:[]}],u=function(e){return function(n){return console.warn("Component "+e+" was not imported, exported, or provided by MDXProvider as global scope"),(0,i.kt)("div",n)}},m=u("FbSuspensePlaceholder"),h=u("FbProfilePhotoHeaderExample"),g={toc:p};function f(e){var n=e.components,t=(0,r.Z)(e,s);return(0,i.kt)("wrapper",(0,a.Z)({},g,t,{components:n,mdxType:"MDXLayout"}),(0,i.kt)("p",null,"When rendering cached data in Relay, it is possible to perform partial rendering. We define ",(0,i.kt)("em",{parentName:"p"},'"partial rendering"')," as the ability to immediately render a query that is partially cached. That is, parts of the query might be missing, but parts of the query might already be cached. In these cases, we want to be able to immediately render the parts of the query that are cached, without waiting on the full query to be fetched."),(0,i.kt)("p",null,"This can be useful in scenarios where we want to render a screen or a page as fast as possible, and we know that some of the data for that page is already cached so we can skip a loading state. For example, take the profile page: it is very likely that the user's name has already been cached at some point during usage of the app, so when visiting a profile page, if the name of the user is cached, we'd like to render immediately, even if the rest of the data for the profile page isn't available yet."),(0,i.kt)("h3",{id:"fragments-as-boundaries-for-partial-rendering"},"Fragments as boundaries for partial rendering"),(0,i.kt)("p",null,"To do this, we rely on the ability of fragment components to ",(0,i.kt)("em",{parentName:"p"},"suspend")," (see the ",(0,i.kt)("a",{parentName:"p",href:"../../rendering/loading-states/"},"Loading States with Suspense")," section). ",(0,i.kt)("em",{parentName:"p"},"A fragment component will suspend")," ",(0,i.kt)("em",{parentName:"p"},"if")," ",(0,i.kt)("em",{parentName:"p"},"any of the data it declared locally is missing during render, and is currently being fetched.")," Specifically, it will suspend until the data it requires is fetched, that is, until the query it belongs to (its ",(0,i.kt)("em",{parentName:"p"},"parent query"),") is fetched."),(0,i.kt)("p",null,"Let's explain what this means with an example. Say we have the following fragment component:"),(0,i.kt)("pre",null,(0,i.kt)("code",{parentName:"pre",className:"language-js"},"/**\n * UsernameComponent.react.js\n *\n * Fragment Component\n */\n\nimport type {UsernameComponent_user$key} from 'UsernameComponent_user.graphql';\n\nconst React = require('React');\nconst {graphql, useFragment} = require('react-relay');\n\ntype Props = {\n  user: UsernameComponent_user$key,\n};\n\nfunction UsernameComponent(props: Props) {\n  const user = useFragment(\n    graphql`\n      fragment UsernameComponent_user on User {\n        username\n      }\n    `,\n    props.user,\n  );\n  return (...);\n}\n\nmodule.exports = UsernameComponent;\n")),(0,i.kt)("p",null,"And we have the following query component,  which queries for some data, and also includes the fragment above:"),(0,i.kt)("pre",null,(0,i.kt)("code",{parentName:"pre",className:"language-javascript"},"/**\n * AppTabs.react.js\n *\n * Query Loader Component\n */\n\n // ....\n\n  const onSelectHomeTab = () => {\n    loadHomeTabQuery({id: '4'}, {fetchPolicy: 'store-or-network'});\n  }\n\n // ...\n\n/**\n * HomeTab.react.js\n *\n * Query Component\n */\n\nconst React = require('React');\nconst {graphql, usePreloadedQuery} = require('react-relay');\n\nconst UsernameComponent = require('./UsernameComponent.react');\n\nfunction HomeTab(props: Props) {\n  const data = usePreloadedQuery<AppQuery>(\n    graphql`\n      query HomeTabQuery($id: ID!) {\n        user(id: $id) {\n          name\n          ...UsernameComponent_user\n        }\n      }\n    `,\n    props.queryRef,\n  );\n\n  return (\n    <>\n      <h1>{data.user?.name}</h1>\n      <UsernameComponent user={data.user} />\n    </>\n  );\n}\n")),(0,i.kt)("p",null,"Say that when this ",(0,i.kt)("inlineCode",{parentName:"p"},"HomeTab")," component is rendered, we've already previously fetched ",(0,i.kt)("em",{parentName:"p"},"(",(0,i.kt)("em",{parentName:"em"},"only"),")")," the ",(0,i.kt)("inlineCode",{parentName:"p"},"name")," for the ",(0,i.kt)("inlineCode",{parentName:"p"},"User")," with ",(0,i.kt)("inlineCode",{parentName:"p"},"{id: 4}"),", and it is locally cached in the Relay Store."),(0,i.kt)("p",null,"If we attempt to render the query with a ",(0,i.kt)("inlineCode",{parentName:"p"},"fetchPolicy")," that allows reusing locally cached data (",(0,i.kt)("inlineCode",{parentName:"p"},"'store-or-network'"),", or ",(0,i.kt)("inlineCode",{parentName:"p"},"'store-and-network'"),"), the following will occur:"),(0,i.kt)("ul",null,(0,i.kt)("li",{parentName:"ul"},"The query will check if any of its locally required data is missing. In this case, ",(0,i.kt)("em",{parentName:"li"},"it isn't"),". *",(0,i.kt)("em",{parentName:"li"}," Specifically, the query is only directly querying for the ",(0,i.kt)("inlineCode",{parentName:"em"},"name"),", and the name "),"is",(0,i.kt)("em",{parentName:"li"}," available, so as far as the query is concerned, none of the data it requires to render "),"itself",(0,i.kt)("em",{parentName:"li"}," is missing. This is important to keep in mind: when rendering a query, we eagerly read out data and render the tree, instead of blocking rendering the entire tree until "),"all",(0,i.kt)("em",{parentName:"li"}," of the data for the query  (i.e. including nested fragments) is fetched. As we render, "),"we will consider data to be missing for a component if the data it declared locally is missing, i.e. if any data required to render the current component is missing, and ",(0,i.kt)("em",{parentName:"li"},"not")," if data for descendant components is missing.*"),(0,i.kt)("li",{parentName:"ul"},"Given that the query doesn't have any data missing, it will render, and then attempt to render the child ",(0,i.kt)("inlineCode",{parentName:"li"},"UsernameComponent"),"."),(0,i.kt)("li",{parentName:"ul"},"When the ",(0,i.kt)("inlineCode",{parentName:"li"},"UsernameComponent")," attempts to render the ",(0,i.kt)("inlineCode",{parentName:"li"},"UsernameComponent_user")," fragment, it will notice that some of the data required to render itself is missing; specifically, the ",(0,i.kt)("inlineCode",{parentName:"li"},"username")," is missing. At this point, since ",(0,i.kt)("inlineCode",{parentName:"li"},"UsernameComponent")," has missing data, it will suspend rendering until the network request completes. Note that regardless of which ",(0,i.kt)("inlineCode",{parentName:"li"},"fetchPolicy")," you choose, a network request will always be started if any piece of data for the full query, i.e. including fragments, is missing.")),(0,i.kt)("p",null,"At this point, when ",(0,i.kt)("inlineCode",{parentName:"p"},"UsernameComponent")," suspends due to the missing ",(0,i.kt)("inlineCode",{parentName:"p"},"username"),", ideally we should still be able to render the ",(0,i.kt)("inlineCode",{parentName:"p"},"User"),"'s ",(0,i.kt)("inlineCode",{parentName:"p"},"name")," immediately, since it's locally cached. However, since we aren't using a ",(0,i.kt)("inlineCode",{parentName:"p"},"Suspense")," component to catch the fragment's suspension, the suspension will bubble up and the entire ",(0,i.kt)("inlineCode",{parentName:"p"},"App")," component will be suspended."),(0,i.kt)("p",null,"In order to achieve the desired effect of rendering the ",(0,i.kt)("inlineCode",{parentName:"p"},"name")," when it's available even if the ",(0,i.kt)("inlineCode",{parentName:"p"},"username"),"  is missing, we just need to wrap the ",(0,i.kt)("inlineCode",{parentName:"p"},"UsernameComponent")," in ",(0,i.kt)("inlineCode",{parentName:"p"},"Suspense,")," to ",(0,i.kt)("em",{parentName:"p"},"allow")," the other parts of ",(0,i.kt)("inlineCode",{parentName:"p"},"App")," to continue rendering:"),(0,i.kt)("pre",null,(0,i.kt)("code",{parentName:"pre",className:"language-js"},"/**\n * HomeTab.react.js\n *\n * Query Component\n */\n\nconst React = require('React');\nconst {Suspense} = require('React');\nconst {graphql, usePreloadedQuery} = require('react-relay');\n\nconst UsernameComponent = require('./UsernameComponent.react');\n\n\nfunction HomeTab() {\n  const data = usePreloadedQuery<AppQuery>(\n    graphql`\n      query AppQuery($id: ID!) {\n        user(id: $id) {\n          name\n          ...UsernameComponent_user\n        }\n      }\n    `,\n    props.queryRef,\n  );\n\n  return (\n    <>\n      <h1>{data.user?.name}</h1>\n\n      {/*\n        Wrap the UserComponent in Suspense to allow other parts of the\n        App to be rendered even if the username is missing.\n      */}\n      <Suspense fallback={<LoadingSpinner label=\"Fetching username\" />}>\n        <UsernameComponent user={data.user} />\n      </Suspense>\n    </>\n  );\n}\n")),(0,i.kt)(m,{mdxType:"FbSuspensePlaceholder"}),(0,i.kt)("p",null,"The process that we described above works the same way for nested fragments (i.e. fragments that include other fragments). This means that if the data required to render a fragment is locally cached, the fragment component will be able to render, regardless of whether data for any of its child or descendant fragments is missing. If data for a child fragment is missing, we can wrap it in a ",(0,i.kt)("inlineCode",{parentName:"p"},"Suspense")," component to allow other fragments and parts of the app to continue rendering."),(0,i.kt)("p",null,"As mentioned in our motivating example, this is desirable because it can allows us to skip loading states entirely. More specifically, the ability to render data that is partially available allows us to render intermediate UI states that more closely resemble the final rendered state."),(0,i.kt)(h,{mdxType:"FbProfilePhotoHeaderExample"}),(0,i.kt)(o.Z,{mdxType:"DocsRating"}))}f.isMDXComponent=!0}}]);