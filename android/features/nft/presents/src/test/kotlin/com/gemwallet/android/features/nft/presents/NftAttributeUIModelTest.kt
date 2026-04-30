package com.gemwallet.android.features.nft.presents

import com.wallet.core.primitives.NFTAttribute
import com.wallet.core.primitives.NFTAttributeType
import org.junit.Assert.assertEquals
import org.junit.Test
import java.text.SimpleDateFormat
import java.util.Locale
import java.util.TimeZone

class NftAttributeUIModelTest {
    private val dateFormat = SimpleDateFormat("MMM d, yyyy", Locale.US).apply {
        timeZone = TimeZone.getTimeZone("UTC")
    }

    @Test
    fun value() {
        assertEquals(
            "Sep 9, 2022",
            NftAttributeUIModel(
                NFTAttribute(name = "Created Date", value = "1662714817", valueType = NFTAttributeType.Timestamp),
                dateFormat,
            ).value,
        )
        assertEquals(
            "9",
            NftAttributeUIModel(
                NFTAttribute(name = "Length", value = "9", valueType = NFTAttributeType.String),
                dateFormat,
            ).value,
        )
        assertEquals(
            "abc",
            NftAttributeUIModel(
                NFTAttribute(name = "Created Date", value = "abc", valueType = NFTAttributeType.Timestamp),
                dateFormat,
            ).value,
        )
    }
}
