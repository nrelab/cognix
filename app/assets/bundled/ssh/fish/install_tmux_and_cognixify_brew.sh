brew install tmux

if test $status -eq 0
    tmux -Lcognix -CC
    exit
end
