function REL(deltaX, deltaY)
  return GET(POS.x + deltaX, POS.y + deltaY)
end

function RELX(deltaX)
  return REL(deltaX, 0)
end

function RELY(deltaY)
  return REL(0, deltaY)
end

setmetatable(_G, {
  __index = function(_, index)
    if (index == "POS") then return THIS_POS() end
    return CELL_POS(index)
  end
})

SIN = math.sin
COS = math.cos
TG = math.tan

function CTG(x)
  return COS(x) / SIN(x)
end

TAN = TG
COT = CTG

ASIN = math.asin
ACOS = math.acos
ATAN = math.atan

LOG = math.log

function LN(x)
  return math.log(x)
end

function LOG2(x)
  return math.log(x, 2)
end

LOG10 = math.log10
LG = math.log10

SQRT = math.sqrt
POW = math.pow
EXP = math.exp
