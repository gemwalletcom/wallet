package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.model.TransactionExtended
import com.gemwallet.android.serializer.jsonEncoder
import com.gemwallet.android.testkit.mockAssetTron
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionResourceTypeMetadata
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class TransactionDataAggregateResourceTest {

    private val asset = mockAssetTron()

    @Test
    fun stakeFreezeUsesBandwidthResourceMetadata() {
        val aggregate = createAggregate(
            type = TransactionType.StakeFreeze,
            metadata = resourceMetadata(Resource.Bandwidth),
        )

        assertEquals(Resource.Bandwidth, aggregate.resourceType)
    }

    @Test
    fun stakeUnfreezeUsesEnergyResourceMetadata() {
        val aggregate = createAggregate(
            type = TransactionType.StakeUnfreeze,
            metadata = resourceMetadata(Resource.Energy),
        )

        assertEquals(Resource.Energy, aggregate.resourceType)
    }

    @Test
    fun stakeFreezeMissingMetadataHasNoResourceType() {
        val aggregate = createAggregate(
            type = TransactionType.StakeFreeze,
            metadata = null,
        )

        assertNull(aggregate.resourceType)
    }

    @Test
    fun stakeUnfreezeMalformedMetadataHasNoResourceType() {
        val aggregate = createAggregate(
            type = TransactionType.StakeUnfreeze,
            metadata = """{"resourceType":"invalid"}""",
        )

        assertNull(aggregate.resourceType)
    }

    private fun createAggregate(
        type: TransactionType,
        metadata: String?,
    ) = TransactionDataAggregateImpl(
        TransactionExtended(
            transaction = createTransaction(type = type, metadata = metadata),
            asset = asset,
            feeAsset = asset,
            price = null,
            feePrice = null,
            assets = emptyList(),
        )
    )

    private fun createTransaction(
        type: TransactionType,
        metadata: String?,
    ) = Transaction(
        id = TransactionId(Chain.Tron, "transaction-id"),
        assetId = AssetId(Chain.Tron),
        from = "wallet-address",
        to = "wallet-address",
        contract = null,
        type = type,
        state = TransactionState.Confirmed,
        blockNumber = "123456",
        sequence = null,
        fee = "1000",
        feeAssetId = AssetId(Chain.Tron),
        value = "10000000",
        memo = null,
        direction = TransactionDirection.Outgoing,
        utxoInputs = null,
        utxoOutputs = null,
        metadata = metadata,
        createdAt = 1L,
    )

    private fun resourceMetadata(resource: Resource): String =
        jsonEncoder.encodeToString(TransactionResourceTypeMetadata.serializer(), TransactionResourceTypeMetadata(resource))
}
