package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.domains.perpetual.PerpetualPositionAction
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockPerpetualData
import com.gemwallet.android.testkit.mockPerpetualPosition
import com.gemwallet.android.testkit.mockPerpetualPositionData
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletId
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.WalletId
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

class BuildPerpetualParamsImplTest {

    private val asset = mockAsset()
    private val perpetualData = mockPerpetualData(asset = asset)
    private val ownWalletId = mockWalletId("wallet-own")
    private val otherWalletId = mockWalletId("wallet-other")

    private val ownPosition = mockPerpetualPositionData(
        perpetual = perpetualData.perpetual,
        asset = asset,
        position = mockPerpetualPosition(
            id = "own",
            perpetualId = perpetualData.perpetual.id,
            assetId = asset.id,
            direction = PerpetualDirection.Long,
            marginAmount = 12.0,
        ),
    )
    private val otherWalletPosition = mockPerpetualPositionData(
        perpetual = perpetualData.perpetual,
        asset = asset,
        position = mockPerpetualPosition(
            id = "other",
            perpetualId = perpetualData.perpetual.id,
            assetId = asset.id,
            direction = PerpetualDirection.Short,
            marginAmount = 9_999.0,
        ),
    )

    private val perpetualRepository = mockk<PerpetualRepository> {
        every { getPerpetual(perpetualData.perpetual.id) } returns flowOf(perpetualData)
        every { getPositionByPerpetualId(ownWalletId, perpetualData.perpetual.id) } returns flowOf(ownPosition)
        every { getPositionByPerpetualId(otherWalletId, perpetualData.perpetual.id) } returns flowOf(otherWalletPosition)
    }

    @Test
    fun `reduce builds params from the session wallet position, not from any wallet on this market`() = runTest {
        val own = reduceFor(ownWalletId)
        val other = reduceFor(otherWalletId)

        assertEquals(PerpetualDirection.Long, own.positionDirection)
        assertEquals(PerpetualDirection.Short, other.positionDirection)
        assertEquals(true, own.available < other.available)
    }

    private suspend fun reduceFor(walletId: WalletId): PerpetualPositionAction.Reduce {
        val sessionRepository = mockk<SessionRepository> {
            every { session() } returns MutableStateFlow(mockSession(wallet = mockWallet(id = walletId.id)))
        }
        val subject = BuildPerpetualParamsImpl(
            perpetualRepository = perpetualRepository,
            sessionRepository = sessionRepository,
        )
        val params = requireNotNull(subject.reduce(perpetualData.perpetual.id))
        return params.positionAction as PerpetualPositionAction.Reduce
    }
}
