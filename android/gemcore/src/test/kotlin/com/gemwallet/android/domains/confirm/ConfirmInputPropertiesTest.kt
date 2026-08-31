package com.gemwallet.android.domains.confirm

import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetCosmos
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockDelegation
import com.gemwallet.android.testkit.mockDelegationValidator
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Test
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransferService
import java.math.BigInteger

class ConfirmInputPropertiesTest {

    private val transferService = GemTransferService()

    private val properties = ConfirmInputProperties(GemTransferService())
    private val codec = ConfirmInputCodec(GemTransferService())

    @Test
    fun propertiesMatchConfirmParamsForEveryVariant() {
        variants().forEach { params ->
            val input = params.toConfirmInput()

            assertEquals(params.assetId, properties.assetId(input))
            assertEquals(params.asset, properties.asset(input))
            assertEquals(params.getTransactionType(transferService), properties.transactionType(input))
        }
    }

    @Test
    fun codecRoundTripsEveryVariantThroughTheRouteString() {
        variants().forEach { params ->
            val packed = requireNotNull(codec.pack(params.toConfirmInput())) { "pack failed for $params" }
            val decoded = requireNotNull(codec.unpack(packed)) { "unpack failed for $params" }

            assertEquals(packed, codec.pack(decoded))
        }
    }

    @Test
    fun decodedInputIsNotComparableByEquality() {
        val params = variants().first { it is ConfirmParams.Stake.UndelegateParams }
        val decoded = requireNotNull(codec.unpack(requireNotNull(codec.pack(params.toConfirmInput()))))

        assertNotEquals(params.toConfirmInput(), decoded)
        assertEquals(properties.assetId(params.toConfirmInput()), properties.assetId(decoded))
    }

    @Test
    fun codecProducesTheSameRouteStringAsConfirmParams() {
        variants().forEach { params ->
            assertEquals(params.pack(transferService), codec.pack(params.toConfirmInput()))
        }
    }

    private fun variants(): List<ConfirmParams> {
        val destination = GemRecipient("destination")
        val stakeAsset = mockAssetCosmos()
        val stakeAccount = mockAccount(chain = Chain.Cosmos)
        val validator = mockDelegationValidator(chain = Chain.Cosmos)
        val delegation = mockDelegation(assetId = stakeAsset.id, validator = validator)
        val stakeBuilder = ConfirmParams.Builder(stakeAsset, stakeAccount, BigInteger.TEN)

        return listOf(
            ConfirmParams.Builder(mockAssetEthereum(), mockAccount(chain = Chain.Ethereum), BigInteger.ONE)
                .transfer(destination),
            ConfirmParams.Builder(mockAssetSolanaUSDC(), mockAccount(chain = Chain.Solana), BigInteger.TWO)
                .transfer(destination),
            stakeBuilder.delegate(validator),
            stakeBuilder.undelegate(delegation),
            stakeBuilder.rewards(listOf(validator)),
        )
    }
}
