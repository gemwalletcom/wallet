package com.gemwallet.android.ext

object VersionCheck {
    fun isVersionHigher(new: String, current: String): Boolean {
        val newComponents = new.split(".").mapNotNull { it.toIntOrNull() }
        val currentComponents = current.split(".").mapNotNull { it.toIntOrNull() }

        for ((newComponent, currentComponent) in newComponents.zip(currentComponents)) {
            when {
                newComponent > currentComponent -> return true
                newComponent < currentComponent -> return false
            }
        }

        return newComponents.size > currentComponents.size
    }
}
