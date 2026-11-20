require 'json'

package = JSON.parse(File.read(File.join(__dir__, 'package.json')))

# Respaldo para proyectos iOS con CocoaPods (Capacitor 8 usa Swift Package Manager: ver
# Package.swift). El núcleo es el mismo XCFramework que arma scripts/ios-nucleo.sh.
Pod::Spec.new do |s|
  s.name = 'CapacitorSonidos'
  s.version = package['version']
  s.summary = package['description']
  s.license = package['license']
  s.homepage = 'https://github.com/ellaguno/sesolibre-sonidos'
  s.author = 'SesoLibre'
  s.source = { :git => 'https://github.com/ellaguno/sesolibre-sonidos.git', :tag => package['version'] }
  s.source_files = 'ios/Sources/**/*.swift'
  s.vendored_frameworks = 'ios/Frameworks/SonidosNucleo.xcframework'
  s.frameworks = 'AVFoundation', 'MediaPlayer'
  s.ios.deployment_target = '15.0'
  s.dependency 'Capacitor'
  s.static_framework = true
  s.swift_version = '5.1'
end
