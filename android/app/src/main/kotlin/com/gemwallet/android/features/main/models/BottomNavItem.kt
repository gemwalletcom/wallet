package com.gemwallet.android.features.main.models

import androidx.compose.ui.graphics.vector.ImageVector

data class BottomNavItem(
    val label: String,
    val icon: ImageVector,
    val route: String,
    val badge: String? = null,
    val testTag: String,
)
