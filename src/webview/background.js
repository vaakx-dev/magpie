((enabled) => {
  const key = Symbol.for("magpie.backgroundPlay")
  const installed = window[key]
  if (installed) {
    installed.enabled = enabled
    return
  }
  if (!enabled) return
  const state = { enabled }
  Object.defineProperty(window, key, { value: state })
  const keepVisible = (property, shown) => {
    const original = Object.getOwnPropertyDescriptor(Document.prototype, property)
    if (!original?.get) return
    Object.defineProperty(Document.prototype, property, {
      configurable: true,
      enumerable: original.enumerable,
      get() {
        return state.enabled ? shown : original.get.call(this)
      },
    })
  }
  keepVisible("hidden", false)
  keepVisible("webkitHidden", false)
  keepVisible("visibilityState", "visible")
  keepVisible("webkitVisibilityState", "visible")
  const swallow = (event) => {
    if (state.enabled) event.stopImmediatePropagation()
  }
  window.addEventListener("visibilitychange", swallow, true)
  window.addEventListener("webkitvisibilitychange", swallow, true)
})
