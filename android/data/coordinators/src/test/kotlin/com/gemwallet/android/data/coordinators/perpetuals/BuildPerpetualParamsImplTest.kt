package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.data.services.gemstone.stores.GemstonePerpetualStore
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockGemPerpetualTransferData
import com.wallet.core.primitives.PerpetualPosition
import uniffi.gemstone.GemPerpetualDetailsServiceInterface
import uniffi.gemstone.GemPerpetualPositionAction
import uniffi.gemstone.GemPerpetualPositionKind
import java.math.BigInteger
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

    private val perpetualStore = mockk<GemstonePerpetualStore> {
        every { observePerpetual(perpetualData.perpetual.id) } returns flowOf(perpetualData)
        every { observePositionByPerpetualId(ownWalletId, perpetualData.perpetual.id) } returns flowOf(ownPosition)
        every { observePositionByPerpetualId(otherWalletId, perpetualData.perpetual.id) } returns flowOf(otherWalletPosition)
    }

    @Test
    fun `reduce builds params from the session wallet position, not from any wallet on this market`() = runTest {
        val own = reduceFor(ownWalletId)
        val other = reduceFor(otherWalletId)

        assertEquals(PerpetualDirection.Long.toGem(), own.data.direction)
        assertEquals(PerpetualDirection.Short.toGem(), other.data.direction)
        assertEquals(true, own.available < other.available)
    }

    private suspend fun reduceFor(walletId: WalletId): GemPerpetualPositionAction.Reduce {
        val getSession = mockk<GetSession> {
            every { this@mockk() } returns MutableStateFlow(mockSession(wallet = mockWallet(id = walletId.id)))
        }
        val subject = BuildPerpetualParamsImpl(
            perpetualStore = perpetualStore,
            getSession = getSession,
            service = mockk<GemPerpetualDetailsServiceInterface> {
                every { positionAction(any(), any(), any(), any()) } answers {
                    val position = thirdArg<String?>()?.decodeJson<PerpetualPosition>()
                    GemPerpetualPositionAction.Reduce(
                        mockGemPerpetualTransferData(direction = requireNotNull(position).direction),
                        BigInteger.valueOf((position.marginAmount * 1_000_000).toLong()),
                    )
                }
            },
        )
        val params = requireNotNull(subject.position(perpetualData.perpetual.id, GemPerpetualPositionKind.Reduce))
        return params.positionAction as GemPerpetualPositionAction.Reduce
    }
}
