package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.model.TransactionExtended
import com.gemwallet.android.serializer.jsonEncoder
import com.gemwallet.android.testkit.mockAssetTron
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionResourceTypeMetadata
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class TransactionDetailsAggregateResourceTest {

    private val asset = mockAssetTron()

    @Test
    fun stakeFreezeAddsBandwidthResourceRow() {
        val aggregate = createAggregate(
            type = TransactionType.StakeFreeze,
            metadata = resourceMetadata(Resource.Bandwidth),
        )

        assertEquals(Resource.Bandwidth, aggregate.resourceType?.data)
        assertTrue(aggregate.detailsItems.anyResource(Resource.Bandwidth))
    }

    @Test
    fun stakeUnfreezeAddsEnergyResourceRow() {
        val aggregate = createAggregate(
            type = TransactionType.StakeUnfreeze,
            metadata = resourceMetadata(Resource.Energy),
        )

        assertEquals(Resource.Energy, aggregate.resourceType?.data)
        assertTrue(aggregate.detailsItems.anyResource(Resource.Energy))
    }

    @Test
    fun stakeFreezeMissingMetadataHasNoResourceRow() {
        val aggregate = createAggregate(
            type = TransactionType.StakeFreeze,
            metadata = null,
        )

        assertNull(aggregate.resourceType)
        assertTrue(aggregate.detailsItems.none { it is TransactionDetailsValue.ResourceType })
    }

    @Test
    fun stakeUnfreezeMalformedMetadataHasNoResourceRow() {
        val aggregate = createAggregate(
            type = TransactionType.StakeUnfreeze,
            metadata = """{"resourceType":"invalid"}""",
        )

        assertNull(aggregate.resourceType)
        assertTrue(aggregate.detailsItems.none { it is TransactionDetailsValue.ResourceType })
    }

    private val TransactionDetailsAggregateImpl.detailsItems: List<TransactionDetailsValue>
        get() = valueGroups[1].items

    private fun List<TransactionDetailsValue>.anyResource(resource: Resource): Boolean =
        any { it is TransactionDetailsValue.ResourceType && it.data == resource }

    private fun createAggregate(
        type: TransactionType,
        metadata: String?,
    ) = TransactionDetailsAggregateImpl(
        data = TransactionExtended(
            transaction = createTransaction(type = type, metadata = metadata),
            asset = asset,
            feeAsset = asset,
            price = null,
            feePrice = null,
            assets = emptyList(),
        ),
        associatedAssets = emptyList(),
        explorer = TransactionDetailsValue.Explorer("https://example.com", "Explorer"),
        currency = Currency.USD,
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
        jsonEncoder.encodeToString(
            TransactionResourceTypeMetadata.serializer(),
            TransactionResourceTypeMetadata(resource),
        )
}
