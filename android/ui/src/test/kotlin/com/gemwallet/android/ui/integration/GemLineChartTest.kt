package com.gemwallet.android.ui.integration

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.unit.dp
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.testkit.mockGemChartBounds
import com.gemwallet.android.ui.components.chart.ChartPoint
import com.gemwallet.android.ui.components.chart.GemLineChart
import com.gemwallet.android.ui.theme.WalletTheme
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class GemLineChartTest {

    @get:Rule
    val composeRule = createComposeRule()

    @Test
    fun `a zero range skips the invalid animation frame`() {
        composeRule.setContent {
            WalletTheme {
                Box(modifier = Modifier.size(200.dp).testTag(CHART_HOST)) {
                    GemLineChart(
                        points = listOf(ChartPoint(0f, 1f), ChartPoint(1f, 1f)),
                        bounds = mockGemChartBounds(yMin = 1.0, yMax = 1.0),
                        isZoomed = false,
                        lineColor = Color.Blue,
                        indexAt = { null },
                        onZoom = { _, _ -> },
                        onPan = {},
                    )
                }
            }
        }

        composeRule.onNodeWithTag(CHART_HOST).assertIsDisplayed()
    }

    private companion object {
        const val CHART_HOST = "chartHost"
    }
}
