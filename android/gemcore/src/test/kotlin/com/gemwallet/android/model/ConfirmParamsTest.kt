package com.gemwallet.android.model

import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetCosmos
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetHyperCoreUBTC
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockAssetTron
import com.gemwallet.android.testkit.mockDelegation
import com.gemwallet.android.testkit.mockDelegationValidator
import com.gemwallet.android.testkit.mockNftAsset
import com.gemwallet.android.testkit.mockPerpetualConfirmData
import com.gemwallet.android.testkit.mockSwapParams
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.ApplicationMetadataSource
import com.wallet.core.primitives.PerpetualType
import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.swap.ApprovalData
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertSame
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemResource
import uniffi.gemstone.GemStakeType
import uniffi.gemstone.GemTransactionInputType
import uniffi.gemstone.TransactionType as GemTransactionType
import java.math.BigInteger

class ConfirmParamsTest {

    @Test
    fun approvalDataMatchesTransactionType() {
        val approval = ApprovalData(token = "token", spender = "spender", value = "1", isUnlimited = false)
        val swap = mockSwapParams(approval = approval)

        assertSame(approval, swap.approvalData(TransactionType.TokenApproval))
        assertNull(swap.approvalData(TransactionType.Swap))
        assertThrows(ConfirmError.TransactionIncorrect::class.java) {
            mockSwapParams().approvalData(TransactionType.TokenApproval)
        }

        val directApproval = ConfirmParams.Builder(mockAssetSolanaUSDC(), mockAccount(chain = Chain.Solana))
            .approval("data", "provider", "contract")
            .approvalData(TransactionType.TokenApproval)
        assertEquals("contract", directApproval?.spender)
    }

    @Test
    fun genericInputPreservesDecodedTransactionType() {
        val approval = ApprovalData(token = "token", spender = "spender", value = "1", isUnlimited = false)
        val params = ConfirmParams.TransferParams.Generic(
            asset = mockAssetEthereum(),
            from = mockAccount(chain = Chain.Ethereum),
            amount = BigInteger.ONE,
            destination = DestinationAddress("destination"),
            memo = "memo",
            inputType = ConfirmParams.TransferParams.InputType.EncodeTransaction,
            isSendable = true,
            metadata = applicationMetadata(),
            data = "0x01",
            gasLimit = "21000",
            decodedTransactionType = TransactionType.TokenApproval,
            approval = approval,
        )

        val input = params.toDto()

        assertTrue(input is GemTransactionInputType.Generic)
        assertEquals(GemTransactionType.TOKEN_APPROVAL, (input as GemTransactionInputType.Generic).extra.transactionType)
        assertEquals("21000", input.extra.gasLimit)
        assertEquals(listOf(1.toByte()), input.extra.data?.toList())
        assertEquals("memo", params.memo)
        assertSame(approval, params.approvalData(TransactionType.TokenApproval))
    }

    @Test
    fun packUnpackRoundTripsEveryVariant() {
        val destination = DestinationAddress("destination")
        val nativeAsset = mockAsset()
        val nativeAccount = mockAccount()
        val tokenAsset = mockAssetSolanaUSDC()
        val tokenAccount = mockAccount(chain = Chain.Solana)
        val stakeAsset = mockAssetCosmos()
        val stakeAccount = mockAccount(chain = Chain.Cosmos)
        val validator = mockDelegationValidator(chain = Chain.Cosmos)
        val delegation = mockDelegation(assetId = stakeAsset.id, validator = validator)
        val stakeBuilder = ConfirmParams.Builder(stakeAsset, stakeAccount, BigInteger.TEN)
        val perpetualAsset = mockAssetHyperCoreUBTC()
        val perpetualAccount = mockAccount(chain = Chain.HyperCore)

        val variants = listOf<ConfirmParams>(
            ConfirmParams.TransferParams.Generic(
                asset = mockAssetEthereum(),
                from = mockAccount(chain = Chain.Ethereum),
                amount = BigInteger.ONE,
                destination = destination,
                memo = "memo",
                inputType = ConfirmParams.TransferParams.InputType.EncodeTransaction,
                isSendable = true,
                metadata = applicationMetadata(),
                data = "0x01",
                gasLimit = "21000",
                decodedTransactionType = TransactionType.SmartContractCall,
            ),
            ConfirmParams.Builder(nativeAsset, nativeAccount, BigInteger.ONE).deposit(destination),
            ConfirmParams.Builder(nativeAsset, nativeAccount, BigInteger.ONE).withdrawal(destination),
            ConfirmParams.Builder(tokenAsset, tokenAccount).approval("data", "Provider", "contract"),
            mockSwapParams(),
            ConfirmParams.Builder(nativeAsset, nativeAccount).activate(),
            ConfirmParams.NftParams(
                asset = mockAssetEthereum(),
                from = mockAccount(chain = Chain.Ethereum),
                destination = destination,
                nftAsset = mockNftAsset(),
            ),
            stakeBuilder.delegate(validator),
            stakeBuilder.withdraw(delegation),
            stakeBuilder.undelegate(delegation),
            stakeBuilder.redelegate(
                destinationValidator = mockDelegationValidator(
                    chain = Chain.Cosmos,
                    id = "destination-validator",
                ),
                delegation = delegation,
            ),
            stakeBuilder.rewards(listOf(validator)),
            stakeBuilder.freeze(Resource.Bandwidth),
            stakeBuilder.unfreeze(Resource.Energy),
            ConfirmParams.Builder(perpetualAsset, perpetualAccount, BigInteger.ONE).perpetual(
                PerpetualType.Open(mockPerpetualConfirmData()),
            ),
        )

        variants.forEach { original ->
            val packed = original.pack()
            assertNotNull(packed)
            val decoded = ConfirmParams.unpack(requireNotNull(packed))
            assertNotNull(decoded)
            assertEquals(original::class, decoded!!::class)
            assertEquals(packed, decoded.pack())
        }
    }

    @Test
    fun freezeMapsToGemFreezeStakeType() {
        val params = ConfirmParams.Builder(
            asset = mockAssetTron(),
            from = mockAccount(chain = Chain.Tron),
            amount = BigInteger.TEN,
        ).freeze(Resource.Bandwidth)

        val inputType = params.toDto()

        assertTrue(inputType is GemTransactionInputType.Stake)
        val stakeType = (inputType as GemTransactionInputType.Stake).stakeType
        assertTrue(stakeType is GemStakeType.Freeze)
        assertEquals(GemResource.BANDWIDTH, (stakeType as GemStakeType.Freeze).resource)
    }

    @Test
    fun unfreezeMapsToGemUnfreezeStakeType() {
        val params = ConfirmParams.Builder(
            asset = mockAssetTron(),
            from = mockAccount(chain = Chain.Tron),
            amount = BigInteger.TEN,
        ).unfreeze(Resource.Energy)

        val inputType = params.toDto()

        assertTrue(inputType is GemTransactionInputType.Stake)
        val stakeType = (inputType as GemTransactionInputType.Stake).stakeType
        assertTrue(stakeType is GemStakeType.Unfreeze)
        assertEquals(GemResource.ENERGY, (stakeType as GemStakeType.Unfreeze).resource)
    }

    private fun applicationMetadata() = ApplicationMetadata(
        name = "App",
        description = "Description",
        url = "https://example.com",
        icon = "https://example.com/icon.png",
        source = ApplicationMetadataSource.WalletConnect,
    )
}
