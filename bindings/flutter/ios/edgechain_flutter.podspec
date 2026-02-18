Pod::Spec.new do |s|
  s.name             = 'edgechain_flutter'
  s.version          = '0.1.0'
  s.summary          = 'Flutter binding for EdgeChain — local-first LLM agent runtime.'
  s.description      = <<-DESC
    Flutter binding for EdgeChain. Run GGUF language models entirely on-device
    with a ReAct agent loop, command registry, SQLite memory, and local RAG.
  DESC
  s.homepage         = 'https://github.com/edgechain-org/edgechain'
  s.license          = { :file => '../LICENSE' }
  s.author           = { 'EdgeChain Contributors' => 'hello@edgechain.dev' }
  s.source           = { :path => '.' }
  s.source_files     = 'Classes/**/*'
  s.dependency 'Flutter'
  s.platform         = :ios, '13.0'
  s.pod_target_xcconfig = { 'DEFINES_MODULE' => 'YES', 'EXCLUDED_ARCHS[sdk=iphonesimulator*]' => 'i386' }
  s.swift_version    = '5.0'
end
