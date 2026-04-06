package wallet.android.app

import com.gemwallet.android.data.repositories.device.DeviceRepository
import org.junit.Assert.assertEquals
import org.junit.Test
import java.util.Locale

class TestSyncDeviceLocale {

    @Test
    fun testZH() {
        assertEquals("zh-Hans", DeviceRepository.getLocale(Locale.forLanguageTag("zh")))
        assertEquals("zh-Hans", DeviceRepository.getLocale(Locale.forLanguageTag("zh-CN")))
        assertEquals("zh-Hans", DeviceRepository.getLocale(Locale.forLanguageTag("zh-TW")))
    }

    @Test
    fun testPT() {
        assertEquals("pt", DeviceRepository.getLocale(Locale.forLanguageTag("pt")))
        assertEquals("pt-BR", DeviceRepository.getLocale(Locale.forLanguageTag("pt-BR")))
    }

    @Test
    fun testEN() {
        assertEquals("en", DeviceRepository.getLocale(Locale.forLanguageTag("en")))
        assertEquals("en", DeviceRepository.getLocale(Locale.forLanguageTag("en-US")))
    }
}
