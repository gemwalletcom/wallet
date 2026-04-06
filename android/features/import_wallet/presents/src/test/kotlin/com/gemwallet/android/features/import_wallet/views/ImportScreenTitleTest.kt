package com.gemwallet.android.features.import_wallet.views

import com.gemwallet.android.ui.R
import com.gemwallet.android.model.ImportType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletType
import org.junit.Assert.assertEquals
import org.junit.Test

class ImportScreenTitleTest {

    @Test
    fun multicoinImport_usesMulticoinTitleResource() {
        val title = importSceneTitle(
            importType = ImportType(walletType = WalletType.Multicoin),
            chainName = "",
        )

        assertEquals(ImportSceneTitle.Resource(R.string.wallet_multicoin), title)
    }

    @Test
    fun chainImport_usesChainNameTitle() {
        val title = importSceneTitle(
            importType = ImportType(walletType = WalletType.Single, chain = Chain.Bitcoin),
            chainName = "Bitcoin",
        )

        assertEquals(ImportSceneTitle.Text("Bitcoin"), title)
    }
}
