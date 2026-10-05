package com.gemwallet.android.features.transfer.presents.integration

import android.content.Context
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.features.transfer.presents.confirm.components.ConfirmErrorInfo
import com.gemwallet.android.features.transfer.viewmodels.confirm.models.ConfirmErrorUIModel
import com.gemwallet.android.testkit.mockGemInfoTopic
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.theme.WalletTheme
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class ConfirmErrorInfoTest {
    @get:Rule
    val composeRule = createComposeRule()

    private val problem = ConfirmErrorUIModel(text = "Error", info = mockGemInfoTopic().infoSheet())
    private var error by mutableStateOf<ConfirmErrorUIModel?>(problem)

    @Test
    fun openSheetStaysWhileARefreshReloadsTheSameProblem() {
        composeRule.setContent {
            WalletTheme {
                ConfirmErrorInfo(
                    error = error,
                    acquireRequest = null,
                    isShowBottomSheetInfo = true,
                    onDismissBottomSheetInfo = {},
                    onDismissAcquire = {},
                    onGetAsset = { _, _ -> },
                )
            }
        }
        val title = requireNotNull(problem.info).sheet.title.string(ApplicationProvider.getApplicationContext<Context>())
        composeRule.onNodeWithText(title).assertIsDisplayed()

        error = null

        composeRule.onNodeWithText(title).assertIsDisplayed()
    }
}
