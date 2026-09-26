package com.gemwallet.android.ui.components.list_item

import android.content.Context
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.text
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockGemFormattedNumber
import com.gemwallet.android.testkit.mockGemWalletRow
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.localization.label
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.style.textStyle
import io.mockk.every
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.BlockExplorerLink
import uniffi.gemstone.GemCopy
import uniffi.gemstone.GemCopyKind
import uniffi.gemstone.GemInfoTitle
import uniffi.gemstone.GemInfoTopic
import uniffi.gemstone.GemLatencyStatus
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemRowMenuItem
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.GemWalletPlaceholder
import uniffi.gemstone.GemWalletSubtitle
import uniffi.gemstone.Latency
import uniffi.gemstone.LatencyType
import java.math.BigInteger

class GemListRowUIModelTest {
    @Test
    fun `latency rows render measurements loading and errors`() {
        every { context.getString(R.string.common_latency_in_ms, *anyVararg()) } returns "125 ms"
        every { context.getString(R.string.errors_error) } returns "Error"
        every { context.getString(R.string.nodes_gem_wallet_node) } returns "Gem Wallet Node"
        every { context.getString(R.string.nodes_stream) } returns "Stream"
        val row = GemListRow.Latency(GemListRowTitle.STREAM, "", "api.gemwallet.com", GemLatencyStatus.Result(Latency(LatencyType.FAST, 125.0)))
        val model = (row.uiModel(context) as GemListRowUIModel.Item).model
        assertEquals("Stream", model.title)
        assertEquals("api.gemwallet.com", model.titleExtra)
        assertEquals("125 ms", model.titleTag)
        assertEquals(ListItemTextStyle.Positive, model.titleTagStyle)

        val loading = (GemListRow.Latency(GemListRowTitle.API, "", "api.gemwallet.com", GemLatencyStatus.Loading).uiModel(context) as GemListRowUIModel.Item).model
        assertEquals("API", loading.title)
        assertEquals(ListItemTagType.Progress, loading.titleTagType)

        val error = (GemListRow.Latency(GemListRowTitle.GEM_WALLET_NODE, " 🇺🇸", "gemnodes.com", GemLatencyStatus.Error).uiModel(context) as GemListRowUIModel.Item).model
        assertEquals("Gem Wallet Node 🇺🇸", error.title)
        assertEquals("Error", error.titleTag)
        assertEquals(ListItemTextStyle.Negative, error.titleTagStyle)
    }

    private val context = mockk<Context>(relaxed = true) {
        every { getString(R.string.settings_website) } returns "Visit Website"
        every { getString(R.string.wallet_copy_address) } returns "Copy Address"
        every { getString(R.string.common_copy) } returns "Copy"
        every { getString(R.string.transaction_view_on, "Etherscan") } returns "View on Etherscan"
    }

    @Test
    fun `menu entries read their titles from core and copy reads Copy`() {
        every { context.getString(R.string.common_wallet) } returns "Wallet"
        val website = GemRowMenuItem.Open(title = GemLocalizedText.ViewOn("Etherscan"), url = "https://etherscan.io/address/0x1")
        val row = GemListRow.Wallet(
            title = GemListRowTitle.WALLET,
            wallet = mockGemWalletRow(id = "wallet-1", name = "Wallet", subtitle = GemWalletSubtitle.Multicoin, placeholder = GemWalletPlaceholder.Multicoin),
            menu = listOf(GemRowMenuItem.Copy(GemCopy(kind = GemCopyKind.Address("ethereum"), value = "0x1", display = "0x1")), website),
        )

        val item = row.uiModel(context) as GemListRowUIModel.Item
        assertEquals("Wallet", item.model.title)
        assertEquals(
            listOf(
                GemListRowMenuItem.Copy("Copy", "0x1"),
                GemListRowMenuItem.Open("View on Etherscan", "https://etherscan.io/address/0x1"),
            ),
            item.menu,
        )
    }

    @Test
    fun `an identifier opens the address core names`() {
        val copy = GemCopy(kind = GemCopyKind.Address("ethereum"), value = "0xcontract", display = "0xcont...ract")
        val contract = GemListRow.Identifier(title = GemListRowTitle.CONTRACT, copy = copy, explorer = null, address = "0xcontract", menu = emptyList()).uiModel(context) as GemListRowUIModel.Item
        val tokenId = GemListRow.Identifier(title = GemListRowTitle.TOKEN_ID, copy = copy, explorer = null, address = null, menu = emptyList()).uiModel(context) as GemListRowUIModel.Item

        assertEquals("0xcontract", contract.address)
        assertEquals(null, tokenId.address)
    }

    @Test
    fun `a validator provider row keeps the address it opens`() {
        val row = GemListRow.Provider(title = GemListRowTitle.VALIDATOR, name = "Validator", contract = "cosmosvaloper1").uiModel(context) as GemListRowUIModel.Provider

        assertEquals("cosmosvaloper1", row.contract)
        assertEquals("Validator", row.model.subtitle)
    }

    @Test
    fun `a lines row reads its first line and the next one below it`() {
        every { context.getString(R.string.perpetual_auto_close) } returns "Auto Close"
        val row = GemListRow.Lines(
            title = GemListRowTitle.AUTO_CLOSE,
            lines = listOf(GemLocalizedText.Text("Take Profit: $65,000"), GemLocalizedText.Text("Stop Loss: $55,000")),
            info = GemInfoTopic.AutoClose,
        )

        val model = (row.uiModel(context) as GemListRowUIModel.Item).model
        assertEquals("Auto Close", model.title)
        assertEquals("Take Profit: $65,000", model.subtitle)
        assertEquals("Stop Loss: $55,000", model.subtitleExtra)
        assertEquals(GemInfoTopic.AutoClose.infoSheet(), model.info)
    }

    @Test
    fun `a ranked row carries its rank tag`() {
        every { context.getString(R.string.asset_market_cap) } returns "Market Cap"

        val model = (GemListRow.Ranked(GemListRowTitle.MARKET_CAP, mockGemFormattedNumber(value = 1.0), "#7").uiModel(context) as GemListRowUIModel.Item).model

        assertEquals("Market Cap", model.title)
        assertEquals("#7", model.titleTag)
    }

    @Test
    fun `an identifier opens the explorer only when core gives a link`() {
        every { context.getString(R.string.asset_contract) } returns "Contract"
        val copy = GemCopy(kind = GemCopyKind.Address("ethereum"), value = "0xdAC17F958D2ee523a2206206994597C13D831ec7", display = "0xdAC1...1ec7")
        val explorer = BlockExplorerLink(name = "Etherscan", link = "https://etherscan.io/token/0xdAC17F958D2ee523a2206206994597C13D831ec7")

        val linked = GemListRow.Identifier(title = GemListRowTitle.CONTRACT, copy = copy, explorer = explorer, address = null, menu = emptyList()).uiModel(context) as GemListRowUIModel.Item
        val plain = GemListRow.Identifier(title = GemListRowTitle.CONTRACT, copy = copy, explorer = null, address = null, menu = emptyList()).uiModel(context) as GemListRowUIModel.Item

        assertEquals("Contract", linked.model.title)
        assertEquals("0xdAC1...1ec7", linked.model.subtitle)
        assertEquals(explorer.link, linked.url)
        assertNull(plain.url)
    }

    @Test
    fun `an all time row shows its change in the change's tone`() {
        val change = mockGemFormattedNumber(value = -12.0, unit = GemNumberUnit.Percent).copy(tone = GemValueTone.NEGATIVE)
        val row = GemListRow.AllTime(title = GemListRowTitle.ALL_TIME_HIGH, value = mockGemFormattedNumber(value = 100.0), date = 0L, change = change)

        val model = (row.uiModel(context) as GemListRowUIModel.Item).model

        assertEquals(mockGemFormattedNumber(value = 100.0).text(), model.subtitle)
        assertEquals(change.text(), model.subtitleExtra)
        assertEquals(GemValueTone.NEGATIVE.textStyle(), model.subtitleExtraStyle)
    }

    @Test
    fun `a below minimum amount names the network the minimum and the buy action`() {
        every { context.getString(R.string.info_minimum_amount_description, "**Bitcoin**", "**0.0005 BTC**") } returns "Minimum"
        every { context.getString(R.string.asset_buy_asset, "BTC") } returns "Buy BTC"

        val sheet = GemInfoTopic.MinimumAmount(mockAsset(name = "Bitcoin", symbol = "BTC", decimals = 8).toGem(), BigInteger("50000")).infoSheet().sheet

        assertEquals(GemInfoTitle.MinimumAmount, sheet.title)
        assertEquals("Minimum", sheet.description.string(context))
        assertEquals("Buy BTC", sheet.action?.label(context))
    }

    @Test
    fun `a missing swap quote opens the no quote sheet`() {
        assertEquals(GemInfoTitle.NoQuote, GemInfoTopic.NoQuote.infoSheet().sheet.title)
    }
}
