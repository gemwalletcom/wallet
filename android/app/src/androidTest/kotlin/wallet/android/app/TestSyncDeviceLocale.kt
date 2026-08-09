package wallet.android.app

import com.gemwallet.android.data.repositories.device.DeviceRepository
import com.wallet.core.primitives.DeviceLocale
import org.junit.Assert.assertEquals
import org.junit.Test
import java.util.Locale

class TestSyncDeviceLocale {

    @Test
    fun getDeviceLocale() {
        val locales = listOf(
            "en" to DeviceLocale.EN,
            "en-US" to DeviceLocale.EN,
            "in-ID" to DeviceLocale.ID,
            "iw-IL" to DeviceLocale.HE,
            "pt" to DeviceLocale.PtBR,
            "pt-BR" to DeviceLocale.PtBR,
            "tl-PH" to DeviceLocale.FIL,
            "zh" to DeviceLocale.ZhHans,
            "zh-CN" to DeviceLocale.ZhHans,
            "zh-SG" to DeviceLocale.ZhHans,
            "zh-TW" to DeviceLocale.ZhHant,
            "zh-HK" to DeviceLocale.ZhHant,
            "zh-MO" to DeviceLocale.ZhHant,
        )

        locales.forEach { (languageTag, expected) ->
            assertEquals(expected, DeviceRepository.getDeviceLocale(Locale.forLanguageTag(languageTag)))
        }

        assertEquals(DeviceLocale.EN, DeviceRepository.getDeviceLocale(Locale.forLanguageTag("af-ZA")))
    }
}
