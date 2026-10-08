global fs_settings, gResXtra

on setupResXtra
  fs_settings = [:]
end

on setFullscreen bool
  if fs_settings.voidp then
    return 
  end if
  if fs_settings[#fullscreen] <> bool then
    fs_settings[#fullscreen] = bool
    if bool then
      pfSetFullscreen(1)
    else
      pfSetFullscreen(0)
    end if
  end if
end
