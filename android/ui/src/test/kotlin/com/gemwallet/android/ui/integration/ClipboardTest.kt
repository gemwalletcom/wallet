package com.gemwallet.android.ui.integration

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.os.Build
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.ui.components.clipboard.clear
import com.gemwallet.android.ui.components.clipboard.getPlainText
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config

@RunWith(AndroidJUnit4::class)
class ClipboardTest {
    private val clipboard = ApplicationProvider.getApplicationContext<Context>().getSystemService(ClipboardManager::class.java)

    @Test
    @Config(sdk = [Build.VERSION_CODES.P])
    fun aClearedClipboardHasNothingToPaste() {
        clipboard.setPrimaryClip(ClipData.newPlainText("", "abandon"))
        assertEquals("abandon", clipboard.getPlainText())

        clipboard.clear()

        assertNull(clipboard.getPlainText())
    }
}
