package app.magpie.browser.browser

import android.content.Context
import android.view.MotionEvent
import androidx.swiperefreshlayout.widget.SwipeRefreshLayout
import org.mozilla.geckoview.GeckoResult
import org.mozilla.geckoview.GeckoSession
import org.mozilla.geckoview.GeckoView
import org.mozilla.geckoview.PanZoomController
import org.mozilla.geckoview.PanZoomController.InputResultDetail

private fun InputResultDetail.byPage(): Boolean =
    handledResult() == PanZoomController.INPUT_RESULT_HANDLED || handledResult() == PanZoomController.INPUT_RESULT_UNHANDLED

private fun InputResultDetail.atTop(): Boolean = scrollableDirections() and PanZoomController.SCROLLABLE_FLAG_TOP == 0

private fun InputResultDetail.overscrollsVertically(): Boolean =
    overscrollDirections() and PanZoomController.OVERSCROLL_FLAG_VERTICAL != 0

private fun InputResultDetail.pullsPastTop(): Boolean = byPage() && atTop() && overscrollsVertically()

class PageView(context: Context) : GeckoView(context) {
    private var gesture: GeckoResult<InputResultDetail>? = null

    init {
        isNestedScrollingEnabled = true
        importantForAutofill = IMPORTANT_FOR_AUTOFILL_YES
    }

    override fun onTouchEvent(event: MotionEvent): Boolean {
        val result = onTouchEventForDetailResult(event)
        when (event.actionMasked) {
            MotionEvent.ACTION_DOWN -> hold(result)
            MotionEvent.ACTION_POINTER_DOWN -> keep()
        }
        return session != null
    }

    private fun keep() {
        gesture = null
        parent?.requestDisallowInterceptTouchEvent(true)
    }

    private fun hold(result: GeckoResult<InputResultDetail>) {
        keep()
        gesture = result
        result.accept { detail ->
            if (gesture === result && detail?.pullsPastTop() == true) parent?.requestDisallowInterceptTouchEvent(false)
        }
    }
}

class PageLayout(context: Context) : SwipeRefreshLayout(context) {
    val page = PageView(context)

    init {
        addView(page)
    }

    fun paint(circle: Int, spinner: Int) {
        setProgressBackgroundColorSchemeColor(circle)
        setColorSchemeColors(spinner)
    }

    fun show(session: GeckoSession) {
        if (page.session == session) return
        page.releaseSession()
        page.setSession(session)
    }

    fun hide() {
        page.releaseSession()
    }

    fun reloadOnPull(session: GeckoSession) = setOnRefreshListener {
        session.reload()
        isRefreshing = false
    }
}
