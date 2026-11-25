package uniffi.gemstone

import com.sun.jna.Pointer
import com.sun.jna.Structure
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import java.io.File
import java.util.zip.ZipFile

class ForeignFutureResultStructsTest {

    @Test
    fun initializeForeignFutureResultStructs_sizesEveryGeneratedResultStruct() {
        val moduleClasses = File(UniffiForeignFutureResultVoid::class.java.protectionDomain!!.codeSource.location.toURI())
        val resultStructs = ZipFile(moduleClasses).use { archive ->
            archive.entries().asSequence()
                .map { it.name.substringAfterLast('/') }
                .filter { it.startsWith("UniffiForeignFutureResult") && it.endsWith("\$UniffiByValue.class") }
                .map { Class.forName("uniffi.gemstone.${it.removeSuffix(".class")}").asSubclass(Structure::class.java) }
                .toList()
        }
        val typeInfo = Structure::class.java.getDeclaredMethod("getTypeInfo").apply { isAccessible = true }

        initializeForeignFutureResultStructs()

        val unsized = resultStructs.filter { struct ->
            val descriptor = typeInfo.invoke(Structure.newInstance(struct)) as Pointer
            descriptor.getNativeLong(0).toLong() == 0L
        }
        assertTrue(resultStructs.isNotEmpty())
        assertEquals(emptyList<Class<*>>(), unsized)
    }
}
