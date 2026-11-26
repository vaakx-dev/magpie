package app.magpie.browser.browser

enum class Step { Out, Reset, In }

private const val DEFAULT_PERCENT = 100
private const val MIN_PERCENT = 50
private const val MAX_PERCENT = 200
private const val STEP_PERCENT = 10

@JvmInline
value class Zoom(val percent: Int) {
    val factor: Double get() = percent / 100.0

    fun step(step: Step): Zoom = when (step) {
        Step.Out -> Zoom((percent - STEP_PERCENT).coerceAtLeast(MIN_PERCENT))
        Step.Reset -> DEFAULT
        Step.In -> Zoom((percent + STEP_PERCENT).coerceAtMost(MAX_PERCENT))
    }

    override fun toString(): String = "$percent%"

    companion object {
        val DEFAULT = Zoom(DEFAULT_PERCENT)
    }
}
