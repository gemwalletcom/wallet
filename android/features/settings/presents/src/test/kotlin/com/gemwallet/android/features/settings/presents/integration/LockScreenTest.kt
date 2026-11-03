package com.gemwallet.android.features.settings.presents.integration

import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.material3.Text
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.assert
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.hasAnyDescendant
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.isRoot
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.window.Dialog
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.features.settings.presents.lock.LockScreen
import com.gemwallet.android.features.settings.presents.lock.UnlockedContent
import com.gemwallet.android.ui.theme.WalletTheme
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class LockScreenTest {
    @get:Rule
    val composeRule = createComposeRule()

    private var isUnlocked by mutableStateOf(false)

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

    @Test
    fun aSheetOpenWhenTheAppLocksStaysUnderTheLock() {
        isUnlocked = true
        setLockedContent()

        isUnlocked = false

        val roots = composeRule.onAllNodes(isRoot())
        roots.assertCountEquals(3)
        roots[2].assert(hasAnyDescendant(hasText("Locked")))
    }

    @Test
    fun aSheetRestoredWhileLockedOpensOnlyAfterTheUnlock() {
        setLockedContent()

        val roots = composeRule.onAllNodes(isRoot())
        roots.assertCountEquals(2)
        roots[1].assert(hasAnyDescendant(hasText("Locked")))

        isUnlocked = true

        composeRule.onAllNodes(isRoot()).assertCountEquals(2)
        composeRule.onAllNodes(hasText("Receive")).assertCountEquals(1)
    }

    private fun setLockedContent() {
        composeRule.setContent {
            WalletTheme {
                UnlockedContent(isUnlocked) {
                    BoxWithConstraints { Dialog(onDismissRequest = {}) { Text("Receive") } }
                }
                if (!isUnlocked) {
                    LockScreen(darkTheme = false, onLeave = {}) { Text("Locked") }
                }
            }
        }
    }
}
