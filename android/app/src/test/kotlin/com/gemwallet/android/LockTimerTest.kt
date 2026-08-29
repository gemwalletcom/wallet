package com.gemwallet.android

import android.text.format.DateUtils
import com.gemwallet.android.application.wallet_connect.WalletConnectEvent
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionProposal
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectValidation
import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext
import com.gemwallet.android.data.repositories.config.UserConfig
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class LockTimerTest {

    @Test
    fun shouldRelock_returnsFalseWhenAuthNotRequired() = runTest {
        val timer = lockTimer(authRequired = false, lockIntervalMinutes = 1)
        timer.setPausedAt(0L)

        assertFalse(timer.shouldRelock(now = Long.MAX_VALUE))
    }

    @Test
    fun shouldRelock_returnsFalseWhenWithinLockInterval() = runTest {
        val timer = lockTimer(authRequired = true, lockIntervalMinutes = 1)
        timer.setPausedAt(0L)

        assertFalse(timer.shouldRelock(now = DateUtils.MINUTE_IN_MILLIS))
    }

    @Test
    fun shouldRelock_returnsTrueAfterLockIntervalElapsed() = runTest {
        val timer = lockTimer(authRequired = true, lockIntervalMinutes = 1)
        timer.setPausedAt(0L)

        assertTrue(timer.shouldRelock(now = DateUtils.MINUTE_IN_MILLIS + 1))
    }

    @Test
    fun shouldRelock_returnsTrueImmediatelyWhenIntervalIsZero() = runTest {
        val timer = lockTimer(authRequired = true, lockIntervalMinutes = 0)
        timer.setPausedAt(0L)

        assertTrue(timer.shouldRelock(now = 1L))
    }

    @Test
    fun shouldRelock_returnsFalseWhenWalletConnectRequestActive() = runTest {
        val timer = lockTimer(
            authRequired = true,
            lockIntervalMinutes = 0,
            activeRequest = activeWalletConnectRequest(sessionProposalEvent()),
        )
        timer.setPausedAt(0L)

        assertFalse(timer.shouldRelock(now = Long.MAX_VALUE))
    }

    @Test
    fun shouldRelock_returnsTrueWhenWalletConnectRequestFinished() = runTest {
        val activeRequest = activeWalletConnectRequest(sessionProposalEvent())
        activeRequest.finish()
        val timer = lockTimer(authRequired = true, lockIntervalMinutes = 0, activeRequest = activeRequest)
        timer.setPausedAt(0L)

        assertTrue(timer.shouldRelock(now = Long.MAX_VALUE))
    }

    private fun lockTimer(
        authRequired: Boolean,
        lockIntervalMinutes: Int,
        activeRequest: ActiveWalletConnectRequest = activeWalletConnectRequest(),
    ): LockTimer {
        val userConfig = mockk<UserConfig>()
        every { userConfig.authRequired() } returns authRequired
        every { userConfig.getLockInterval() } returns flowOf(lockIntervalMinutes)
        return LockTimer(userConfig, activeRequest)
    }

    private fun activeWalletConnectRequest(vararg events: WalletConnectEvent) = ActiveWalletConnectRequest(
        events = flowOf(*events),
        scope = CoroutineScope(UnconfinedTestDispatcher()),
    )

    private fun sessionProposalEvent() = WalletConnectEvent.SessionProposal(
        proposal = WalletConnectSessionProposal(
            name = "Dapp",
            description = "",
            url = "https://dapp.test",
            icons = emptyList(),
            requiredNamespaces = emptyMap(),
            optionalNamespaces = emptyMap(),
            proposerPublicKey = "key",
            properties = null,
        ),
        verifyContext = WalletConnectVerifyContext(
            origin = "https://dapp.test",
            validation = WalletConnectValidation.Valid,
            isScam = false,
        ),
    )
}
