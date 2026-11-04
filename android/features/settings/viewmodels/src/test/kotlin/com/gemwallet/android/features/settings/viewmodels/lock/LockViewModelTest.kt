package com.gemwallet.android.features.settings.viewmodels.lock

import com.gemwallet.android.application.security.cases.SecurityPreferences
import com.gemwallet.android.features.settings.viewmodels.lock.models.AuthState
import io.mockk.every
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class LockViewModelTest {

    @Test
    fun relock_clearsStaleAuthStateAndReprompts() {
        val viewModel = lockViewModel(authRequired = true)
        viewModel.onInitialAuth(AuthState.Success)
        viewModel.requestAuth(requestId = 42L)
        val promptCountBefore = viewModel.uiState.value.authPromptRequest

        viewModel.relock()

        val state = viewModel.uiState.value
        assertEquals(AuthState.Required, state.initialAuth)
        assertNull("stale secondary auth must be cleared on relock", state.authState)
        assertTrue(
            "prompt request must bump so the auth LaunchedEffect refires",
            state.authPromptRequest > promptCountBefore,
        )
        assertFalse(
            "the cancelled secondary auth request must not be completable",
            viewModel.completeAuthRequest(requestId = 42L),
        )
    }

    @Test
    fun retryInitialAuth_bumpsPromptWhenStillRequired() {
        val viewModel = lockViewModel(authRequired = true)
        val promptCountBefore = viewModel.uiState.value.authPromptRequest

        viewModel.retryInitialAuth()

        assertEquals(promptCountBefore + 1, viewModel.uiState.value.authPromptRequest)
    }

    @Test
    fun retryInitialAuth_isNoOpAfterUnlock() {
        val viewModel = lockViewModel(authRequired = true)
        viewModel.onInitialAuth(AuthState.Success)
        val stateBefore = viewModel.uiState.value

        viewModel.retryInitialAuth()

        assertEquals(stateBefore, viewModel.uiState.value)
    }

    @Test
    fun returningAfterTheLockPeriodRelocksBeforeReturning() {
        val viewModel = lockViewModel(authRequired = true, lockTimer = mockk { every { shouldRelockOnReturn() } returns true })
        viewModel.onInitialAuth(AuthState.Success)

        assertTrue(viewModel.onReturn())
        assertEquals(AuthState.Required, viewModel.uiState.value.initialAuth)
    }

    @Test
    fun returningWithinTheLockPeriodKeepsTheAppUnlocked() {
        val viewModel = lockViewModel(authRequired = true, lockTimer = mockk { every { shouldRelockOnReturn() } returns false })
        viewModel.onInitialAuth(AuthState.Success)

        assertFalse(viewModel.onReturn())
        assertEquals(AuthState.Success, viewModel.uiState.value.initialAuth)
    }

    private fun lockViewModel(authRequired: Boolean, lockTimer: LockTimer = mockk(relaxed = true)): LockViewModel {
        val securityPreferences = mockk<SecurityPreferences>()
        every { securityPreferences.isLockEnabled() } returns authRequired

        return LockViewModel(
            securityPreferences = securityPreferences,
            lockTimer = lockTimer,
        )
    }
}
