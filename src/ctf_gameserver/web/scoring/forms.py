from django import forms
from django.utils.translation import ugettext_lazy as _

from . import models


class GameControlAdminForm(forms.ModelForm):
    """
    Form for the GameControl object, designed to be used in GameControlAdmin.
    """

    # Ticks longer than 1 hours are possible but don't seem reasonable and would require addtional cleaning
    # logic below
    tick_duration = forms.IntegerField(min_value=1, max_value=3559, help_text=_('Duration of one tick in '
                                                                                'seconds'))

    class Meta:
        model = models.GameControl
        exclude = ('current_tick',)
        help_texts = {
            'services_public': _('Time at which information about the services is public, but the actual '
                                 'game has not started yet'),
            'valid_ticks': _('Number of ticks a flag is valid for'),
            'registration_confirm_text': _('If set, teams will have to confirm to this text (e.g. a link to'
                                           'T&C) when signing up. May contain HTML.'),
            'min_net_number': _('If unset, team IDs will be used as net numbers'),
            'max_net_number': _('(Inclusive) If unset, team IDs will be used as net numbers'),
        }

    def clean_tick_duration(self):
        tick_duration = self.cleaned_data['tick_duration']

        # The timer of the gameserver's Controller component is configured with conditions for the minute
        # and seconds values
        if (tick_duration < 60 and 60 % tick_duration != 0) or \
           (tick_duration > 60 and tick_duration % 60 != 0):
            raise forms.ValidationError(_('The tick duration has to be a multitude of 60!'))

        return tick_duration

    def clean(self):
        services_public = self.cleaned_data['services_public']
        start = self.cleaned_data['start']
        end = self.cleaned_data['end']

        if services_public > start:
            raise forms.ValidationError(_('Services public time must not be after start time'))
        if end <= start:
            raise forms.ValidationError(_('End time must be after start time'))
