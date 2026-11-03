package com.gemwallet.android

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlin.time.Duration.Companion.milliseconds

internal class PrivacyCover(private val scope: CoroutineScope, private val isGemFocused: () -> Boolean, private val isGemLocked: () -> Boolean) {
    private val _isCovered = MutableStateFlow(false)
    val isCovered: StateFlow<Boolean> = _isCovered.asStateFlow()

    var isLockEnabled = false
    private var hasFocus = true
    private var showsGemPrompt = false
    private var promptEndedAway = false
    private var pendingCover: Job? = null
    private val coversGem get() = isLockEnabled && !isGemLocked()

    fun onPromptShown() {
        showsGemPrompt = true
        promptEndedAway = false
    }

    fun onPromptEnded() {
        if (hasFocus) showsGemPrompt = false else promptEndedAway = true
    }

    fun onFocusChanged(focused: Boolean) {
        hasFocus = focused
        pendingCover?.cancel()
        if (focused) {
            if (promptEndedAway) {
                showsGemPrompt = false
                promptEndedAway = false
            }
            _isCovered.value = false
            return
        }
        pendingCover = scope.launch {
            delay(obscureDelay)
            if (coversGem && !showsGemPrompt && !isGemFocused()) _isCovered.value = true
        }
    }

    fun onStop() {
        pendingCover?.cancel()
        if (coversGem) _isCovered.value = true
    }

    fun onReturn() {
        pendingCover?.cancel()
        _isCovered.value = false
    }

    private companion object {
        val obscureDelay = 150.milliseconds
    }
}
