property glob, currentMode, hiscores, stages, unlockedBGs, savedGame, numHighScores, game_mgr, demoMode, unlock, unlocked, expired, daysLeft, save, timebomb, options, initialized, defaults, profiles, currentProfile, prefName, numHiScores, seenItems

on new me, g
  glob = g
  game_mgr = glob.managers.game
  stages = [:]
  numHiScores = 10
  defaults = [:]
  defaults[#fullscreen] = 1
  defaults[#music] = 1
  defaults[#sfx] = 1
  defaults[#profiles] = [:]
  defaults[#seenItems] = []
  c = [[#name: "Margaret", #score: 20000], [#name: "Indra", #score: 15000], [#name: "Jorge", #score: 10000], [#name: "Tamesha", #score: 7500], [#name: "Aziz", #score: 6000], [#name: "Jack", #score: 5000], [#name: "Aiden", #score: 4000], [#name: "Milagros", #score: 3000], [#name: "Wu", #score: 2000], [#name: "Suzie", #score: 1000]]
  ee = [[#name: "Margaret", #score: 2000], [#name: "Indra", #score: 1500], [#name: "Jorge", #score: 1000], [#name: "Tamesha", #score: 750], [#name: "Aziz", #score: 500], [#name: "Jack", #score: 400], [#name: "Aiden", #score: 300], [#name: "Milagros", #score: 200], [#name: "Wu", #score: 100], [#name: "Suzie", #score: 50]]
  defaults[#hiscores] = [#career: c, #endless_easy: ee, #endless_medium: ee.duplicate(), #endless_hard: ee.duplicate()]
  defaults[#savedGame] = [#stage: 1, #level: 1, #bg: "state_1"]
  defaults[#currentProfile] = VOID
  defaults[#options] = [#fullscreen: 1, #music: 1, #sfx: 1]
  defaults[#unlock] = random(999999)
  defaults[#unlockedBGs] = [:]
  return me
end

on init me
  if initialized then
    return 
  end if
  initialized = 1
  timebomb = getConfigData()[#timebomb]
  if voidp(timebomb) then
    timebomb = [#enabled: 0]
  end if
  me.loadState()
end

on lockAll me
  repeat with s = 1 to 5
    i = stages[s]
    repeat with j in i
      j.unlocked = 0
      j.starred = 0
    end repeat
  end repeat
  stages[1][1].unlocked = 1
  savedGame = [#stage: 1, #level: 1]
  seenItems.deleteAll()
  me.saveState()
end

on unlockAll me
  repeat with s = 1 to 4
    if seenItems.getOne(s) = 0 then
      seenItems.add(s)
    end if
    i = stages[s]
    repeat with j in i
      j.unlocked = 1
    end repeat
  end repeat
  savedGame = [#stage: stages.count, #level: stages[stages.count].count]
  me.saveState()
end

on destroy me
  glob = VOID
end

on loadState me
  ls = value(devoid(getPref("lunchprefs")))
  ks = Keys(defaults)
  if voidp(ls) then
    repeat with k in ks
      me[k] = defaults[k]
    end repeat
    me.saveState()
  else
    repeat with k in ks
      v = ls[k]
      if not voidp(v) then
        me[k] = v
        next repeat
      end if
      me[k] = defaults[k]
    end repeat
    unlock = ls[#xlck]
  end if
  hs = [:]
  repeat with i in Keys(defaults.hiscores)
    s = hiscores[i]
    if voidp(s) then
      s = defaults.hiscores[i]
    end if
    hs[i] = s
  end repeat
  hiscores = hs
  if not voidp(currentProfile) then
    me.setCurrentProfile(currentProfile)
  end if
  me.updateOptions()
  if voidp(unlock) then
    unlock = random(999999)
    me.saveState()
  end if
  me.tryCode(unlock)
  return 1
end

on loadProfile me, pfdata
  if voidp(pfdata) then
    return 
  end if
  highestStage = 1
  highestLevel = 1
  cfg = glob.managers.config.game.career.stages
  bgstates = glob.managers.config.gfx
  stages = pfdata.stages
  seenItems = pfdata[#seenItems]
  if not listp(seenItems) then
    seenItems = []
    pfdata[#seenItems] = seenItems
  end if
  repeat with i in Keys(stages)
    st = cfg[i]
    if voidp(cfg[i]) then
      stages.deleteProp(i)
      next repeat
    end if
    myst = stages[i]
    repeat while myst.count > 10
      myst.deleteAt(myst.count)
    end repeat
  end repeat
  repeat with i = 1 to cfg.count
    stname = cfg.getPropAt(i)
    stls = stages[stname]
    if voidp(stls) then
      stls = []
      stages[stname] = stls
    end if
    st = cfg[i]
    repeat with j = 1 to st.numLevels
      if j > stls.count then
        bgs = bgstates[st.id]
        if not voidp(bgs) then
          bgunlock = bgs.states.getPropAt(1)
        end if
        stls.add([#unlocked: 0, #starred: 0])
      end if
      if stls[j].unlocked then
        highestLevel = j
        highestStage = i
      end if
    end repeat
  end repeat
  stages[1][1].unlocked = 1
  if voidp(pfdata[#savedGame]) then
    pfdata[#savedGame] = [#stage: highestStage, #level: highestLevel]
  end if
  savedGame = pfdata[#savedGame]
end

on tryCode me, cd
  if not timebomb.enabled then
    unlocked = 1
    expired = 0
    return 1
  end if
  unlockCode = timebomb[#unlockCode]
  if cd <> unlockCode then
    unlocked = 0
    daysLeft = daysUntil(timebomb.month, timebomb.day, timebomb.year)
    expired = daysLeft < 0
    return 0
  else
    unlocked = 1
    expired = 0
    return 1
  end if
end

on saveState me
  setPref("lunchprefs", devoid(string([#options: options, #profiles: profiles, #unlockedBGs: unlockedBGs, #hiscores: hiscores, #xlck: unlock, #currentProfile: currentProfile])))
end

on starLevel me, Args
  stages[Args.stage][Args.level].starred = 1
  me.saveState()
end

on savegame me, Args
  savedGame = Args
  profiles[currentProfile][#savedGame] = Args
  lvl = stages[Args.stage][Args.level]
  lvl.unlocked = 1
  if (Args.stage >= savedGame.stage) and (Args.level >= savedGame.level) then
    savedGame = Args
  end if
  me.saveState()
end

on checkHighScore me, Args
  m = Args.mode
  sc = hiscores[m]
  if voidp(sc) then
    return 
  end if
  return Args.score > sc[sc.count].score
end

on submitScore me, sc, m, diff
  if sc = 0 then
    return 0
  end if
  if m = #endless then
    if diff = 1 then
      m = #endless_easy
    else
      if diff = 2 then
        m = #endless_medium
      else
        if diff = 3 then
          m = #endless_hard
        end if
      end if
    end if
  end if
  ls = hiscores[m]
  if voidp(ls) then
    return 0
  end if
  repeat with i = 1 to ls.count
    if sc > i then
      exit repeat
    end if
  end repeat
  if i > numHiScores then
    return 0
  end if
  ls.addAt(i, [#name: currentProfile, #score: sc])
  repeat while ls.count > numHiScores
    ls.deleteAt(ls.count)
  end repeat
  me.saveState()
  return i
end

on getProfiles me
  return profiles
end

on getCurrentProfile me
  if not voidp(currentProfile) then
    return currentProfile
  end if
end

on newProfile me, nm
  if not voidp(profiles[nm]) then
    return 0
  end if
  np = [:]
  np[#stages] = [:]
  np[#savedGame] = [#stage: 1, #level: 1]
  np[#name] = nm
  profiles[nm] = np
  return 1
end

on deleteProfile me, nm
  p = profiles[nm]
  if voidp(p) then
    return 
  end if
  if nm = currentProfile then
    currentProfile = VOID
  end if
  profiles.deleteProp(nm)
  me.saveState()
end

on setCurrentProfile me, nm
  currentProfile = nm
  p = profiles[nm]
  if voidp(p) then
    return 0
  end if
  me.loadProfile(p)
  me.saveState()
end

on seenItem me, m
  if seenItems.getOne(m) = 0 then
    seenItems.add(m)
  end if
end

on updateOptions
  if the runMode <> "Author" then
    setFullscreen(options[#fullscreen])
  end if
  SndSetMusicVolume(options[#music] * 100)
  SndSetSFXVolume(options[#sfx] * 100)
end

on unlockBG me, stname, bgnum
  st = integer(unlockedBGs[stname])
  unlockedBGs[stname] = max(st, bgnum)
  me.saveState()
end

on getUnlockedBG me, stname
  return max(1, unlockedBGs[stname])
end
