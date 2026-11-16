package com.gemwallet.android.ui.integration

import android.content.Context
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.junit4.StateRestorationTester
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onAllNodesWithContentDescription
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onFirst
import androidx.compose.ui.test.onLast
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.filters.TransactionsFilter
import com.gemwallet.android.ui.theme.WalletTheme
import com.wallet.core.primitives.WalletType
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.GemTransactionFilter
import uniffi.gemstone.newTransactionsFilterSession

@RunWith(AndroidJUnit4::class)
class TransactionsFilterTest {
    @get:Rule
    val composeRule = createComposeRule()

    private val context = ApplicationProvider.getApplicationContext<Context>()
    private var session by mutableStateOf(newTransactionsFilterSession(emptyList(), WalletType.Multicoin.toGem()))
    private val appliedTypes = mutableListOf<List<GemTransactionFilter>>()

    @Test
    fun typesSheetOpensWithNothingPickedAfterFiltersAreCleared() {
        showFilter()
        pickType(R.string.wallet_swap)
        session = session.onClear()

        pickType(R.string.wallet_stake)

        assertEquals(listOf(GemTransactionFilter.STAKE), appliedTypes.last())
    }

    @Test
    fun typesSheetDropsPicksDismissedWithoutDone() {
        showFilter()
        openTypes()
        composeRule.onNodeWithText(context.getString(R.string.wallet_swap)).performClick()
        composeRule.onAllNodesWithContentDescription(context.getString(R.string.common_close)).onLast().performClick()

        openTypes()
        composeRule.onNodeWithText(context.getString(R.string.common_done)).performClick()

        assertEquals(emptyList<GemTransactionFilter>(), appliedTypes.last())
    }

    @Test
    fun typesSheetKeepsUnappliedPicksAcrossRecreation() {
        val restoration = StateRestorationTester(composeRule)
        restoration.setContent { FilterContent() }
        openTypes()
        composeRule.onNodeWithText(context.getString(R.string.wallet_swap)).performClick()

        restoration.emulateSavedInstanceStateRestore()
        composeRule.onNodeWithText(context.getString(R.string.common_done)).performClick()

        assertEquals(listOf(GemTransactionFilter.SWAPS), appliedTypes.last())
    }

    private fun showFilter() {
        composeRule.setContent { FilterContent() }
    }

    @Composable
    private fun FilterContent() {
        WalletTheme {
            TransactionsFilter(
                isVisible = true,
                filter = session,
                view = session.viewState(),
                onDismissRequest = {},
                onSelectChainsFilter = {},
                onSelectTypesFilter = {
                    appliedTypes.add(it)
                    session = session.onTypes(it)
                },
                onClear = { session = session.onClear() },
            )
        }
    }

    private fun openTypes() {
        composeRule.onAllNodesWithText(context.getString(R.string.filter_types)).onFirst().performClick()
    }

    private fun pickType(label: Int) {
        openTypes()
        composeRule.onNodeWithText(context.getString(label)).performClick()
        composeRule.onNodeWithText(context.getString(R.string.common_done)).performClick()
    }
}
