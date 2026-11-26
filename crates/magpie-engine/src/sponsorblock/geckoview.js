(receive) => (message) => browser.runtime.sendNativeMessage("magpie", message).then(receive)
