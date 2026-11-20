# Reglas de R8 de la app. Capacitor y Media3 traen las suyas (consumerProguardFiles) y el
# plugin conserva Nativo, que el núcleo busca por nombre desde JNI. Esto es defensa
# explícita para lo que se resuelve por reflexión o desde JavaScript.

-keepclasseswithmembers class * {
    @android.webkit.JavascriptInterface <methods>;
}
-keepattributes *Annotation*, Signature, InnerClasses, EnclosingMethod
-keep class com.getcapacitor.** { *; }
-keep @com.getcapacitor.annotation.CapacitorPlugin class * { *; }
-keep class * extends com.getcapacitor.Plugin { *; }
-keep class com.capacitorjs.plugins.** { *; }
-keep class org.apache.cordova.** { *; }

# El plugin entero: el servicio va declarado en el manifiesto y el puente JNI usa nombres.
-keep class com.sesolibre.sonidos.** { *; }

# Números de línea legibles en los fallos de Play Console.
-keepattributes SourceFile,LineNumberTable
-renamesourcefileattribute SourceFile
