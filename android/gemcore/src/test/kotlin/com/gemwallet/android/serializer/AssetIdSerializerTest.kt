package com.gemwallet.android.serializer

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import kotlinx.serialization.encodeToString
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test
import java.io.IOException

class AssetIdSerializerTest {

    @Test
    fun `serializes and deserializes AssetId as identifier string`() {
        val assetId = AssetId(chain = Chain.Ethereum, tokenId = "0xabc")
        val json = jsonEncoder.encodeToString(assetId)

        assertEquals("\"ethereum_0xabc\"", json)
        assertEquals(assetId, jsonEncoder.decodeFromString<AssetId>(json))
        assertEquals("\"ethereum\"", jsonEncoder.encodeToString(AssetId(Chain.Ethereum)))
        assertEquals("\"ethereum_SomeTOken\"", jsonEncoder.encodeToString(AssetId(Chain.Ethereum, "SomeTOken")))
        assertEquals(AssetId(Chain.Ethereum), jsonEncoder.decodeFromString<AssetId>("ethereum"))
        assertEquals(AssetId(Chain.Ethereum, "0xABSDEEF"), jsonEncoder.decodeFromString<AssetId>("ethereum_0xABSDEEF"))
    }

    @Test
    fun `deserializes AssetId from object payload`() {
        val json = """{"chain":"ethereum","tokenId":"0xabc"}"""

        val decoded = jsonEncoder.decodeFromString<AssetId>(json)

        assertEquals(AssetId(chain = Chain.Ethereum, tokenId = "0xabc"), decoded)
        assertEquals(AssetId(Chain.Ethereum), jsonEncoder.decodeFromString<AssetId>("""{"chain":"Ethereum"}"""))
        assertEquals(
            AssetId(Chain.Ethereum, "0xABSDEEF"),
            jsonEncoder.decodeFromString<AssetId>("""{"chain":"Ethereum","tokenId":"0xABSDEEF"}"""),
        )
        assertThrows(IOException::class.java) {
            jsonEncoder.decodeFromString<AssetId>("""{"chain":"FooChain","tokenId":"0xABSDEEF"}""")
        }
    }
}
