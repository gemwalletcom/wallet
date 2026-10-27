package com.gemwallet.android.features.settings.presents.integration

import androidx.compose.material3.Text
import androidx.compose.ui.test.assert
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.hasAnyDescendant
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.isRoot
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.window.Dialog
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.features.settings.presents.lock.LockScreen
import com.gemwallet.android.ui.theme.WalletTheme
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class LockScreenTest {
    @get:Rule
    val composeRule = createComposeRule()

    @Test
    fun theLockIsTheTopWindowAboveAnOpenSheet() {
        composeRule.setContent {
            WalletTheme {
                Dialog(onDismissRequest = {}) { Text("Receive") }
                LockScreen(darkTheme = false, onLeave = {}) { Text("Locked") }
            }
        }

        val roots = composeRule.onAllNodes(isRoot())
        roots.assertCountEquals(3)
        roots[2].assert(hasAnyDescendant(hasText("Locked")))
    }
}
