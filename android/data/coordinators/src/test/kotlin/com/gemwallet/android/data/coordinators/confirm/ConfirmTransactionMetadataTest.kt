package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.serializer.jsonEncoder
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetTron
import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.TransactionResourceTypeMetadata
import org.junit.Assert.assertEquals
import org.junit.Test
import java.math.BigInteger

class ConfirmTransactionMetadataTest {

    private val asset = mockAssetTron()
    private val account = mockAccount(asset.id.chain)

    @Test
    fun freezeEncodesResourceMetadata() {
        val params = ConfirmParams.Builder(asset, account, BigInteger.TEN)
            .freeze(Resource.Bandwidth)

        val metadata = requireNotNull(params.toTransactionMetadataJson())
        val decoded = jsonEncoder.decodeFromString(
            TransactionResourceTypeMetadata.serializer(),
            metadata,
        )

        assertEquals(Resource.Bandwidth, decoded.resourceType)
    }

    @Test
    fun unfreezeEncodesResourceMetadata() {
        val params = ConfirmParams.Builder(asset, account, BigInteger.TEN)
            .unfreeze(Resource.Energy)

        val metadata = requireNotNull(params.toTransactionMetadataJson())
        val decoded = jsonEncoder.decodeFromString(
            TransactionResourceTypeMetadata.serializer(),
            metadata,
        )

        assertEquals(Resource.Energy, decoded.resourceType)
    }
}
