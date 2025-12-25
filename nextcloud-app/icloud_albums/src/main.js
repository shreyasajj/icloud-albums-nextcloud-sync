import Vue from 'vue'
import App from './App.vue'

// eslint-disable-next-line
__webpack_nonce__ = btoa(OC.requestToken)

Vue.prototype.OC = OC
Vue.prototype.OCA = OCA

export default new Vue({
    el: '#app',
    render: h => h(App),
})
