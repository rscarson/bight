function POSX()
  x, _ = POS()
  return x
end

function POSY()
  _, y = POS()
  return y
end

function REL(deltaX, deltaY)
  x, y = POS()
  return GET(x + deltaX, y + deltaY)
end

function RELX(deltaX)
  return REL(0, deltaX)
end

function RELY(deltaY)
  return REL(0, deltaY)
end

setmetatable(_G, {
  __index = function(_, index)
    return GET(index)
  end
})
